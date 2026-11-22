use fuser::{
    Errno, FileAttr, FileHandle, FileType, Filesystem, FopenFlags, Generation, INodeNo, LockOwner,
    OpenFlags, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatx,
    ReplyWrite, ReplyXattr, Request, StatxMask, TimeOrNow, WriteFlags,
};
use indexmap::IndexSet;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use tempfile::{TempDir, tempdir};

static GLOBAL_MOCK_MOUNT: Mutex<Option<(fuser::BackgroundSession, TempDir)>> = Mutex::new(None);

pub fn spawn_mock_fs(source_path: PathBuf) -> PathBuf {
    let mock_fs_dir = tempdir().unwrap();
    let mock_fs_path = mock_fs_dir.path().to_path_buf();

    let mock_fs = MockSysfs::new(source_path);
    let handle = fuser::spawn_mount(mock_fs, &mock_fs_path, &fuser::Config::default()).unwrap();

    *GLOBAL_MOCK_MOUNT.lock().unwrap() = Some((handle, mock_fs_dir));

    mock_fs_path
}

/// Files which are writeable as-is (reads will return last write's content)
const WRITEABLE_FILES: &[&str] = &["power_dpm_force_performance_level"];

pub struct MockSysfs {
    source_path: PathBuf,
    inodes: Mutex<IndexSet<PathBuf>>,
    pub writes: Arc<Mutex<Vec<WrittenEntry>>>,
}

pub struct WrittenEntry {
    pub path: PathBuf,
    pub contents: Vec<u8>,
}

impl MockSysfs {
    pub fn new(source_path: PathBuf) -> Self {
        MockSysfs {
            source_path,
            inodes: Mutex::new(IndexSet::from([PathBuf::new()])),
            writes: Arc::default(),
        }
    }

    fn path(&self, ino: INodeNo) -> Result<PathBuf, Errno> {
        let index = usize::try_from(u64::from(ino) - 1).unwrap();
        self.inodes
            .lock()
            .unwrap()
            .get_index(index)
            .cloned()
            .ok_or(Errno::ENOENT)
    }

    fn inode(&self, path: &Path) -> INodeNo {
        let mut inodes = self.inodes.lock().unwrap();
        let (index, _) = inodes.insert_full(path.to_path_buf());
        INodeNo(index as u64 + 1)
    }

    fn attr(&self, path: &Path) -> Result<FileAttr, Errno> {
        let metadata = fs::symlink_metadata(self.source_path.join(path))?;
        let mut attr = FileAttr::try_from(&metadata)?;
        attr.ino = self.inode(path);
        Ok(attr)
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Errno> {
        if path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| WRITEABLE_FILES.contains(&name))
        {
            let writes = self.writes.lock().unwrap();
            if let Some(entry) = writes.iter().rfind(|write| write.path == path) {
                return Ok(entry.contents.clone());
            }
        }
        Ok(fs::read(self.source_path.join(path))?)
    }
}

impl Filesystem for MockSysfs {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        match self.path(parent).and_then(|p| self.attr(&p.join(name))) {
            Ok(attr) => reply.entry(&Duration::ZERO, &attr, Generation(0)),
            Err(err) => reply.error(err),
        }
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        match self.path(ino).and_then(|p| self.attr(&p)) {
            Ok(attr) => reply.attr(&Duration::ZERO, &attr),
            Err(err) => reply.error(err),
        }
    }

    // Called on truncation when opening files for writing
    fn setattr(
        &self,
        req: &Request,
        ino: INodeNo,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<FileHandle>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<fuser::BsdFileFlags>,
        _kill_suid_gid: bool,
        reply: ReplyAttr,
    ) {
        self.getattr(req, ino, fh, reply);
    }

    fn open(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _flags: OpenFlags,
        _kill_suid_gid: bool,
        reply: ReplyOpen,
    ) {
        reply.opened(FileHandle(0), FopenFlags::FOPEN_DIRECT_IO);
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyData,
    ) {
        match self.path(ino).and_then(|p| self.read_file(&p)) {
            Ok(data) => {
                let start = usize::try_from(offset).unwrap().min(data.len());
                let end = (start + size as usize).min(data.len());
                reply.data(&data[start..end]);
            }
            Err(err) => reply.error(err),
        }
    }

    fn write(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        _offset: u64,
        data: &[u8],
        _write_flags: WriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyWrite,
    ) {
        match self.path(ino) {
            Ok(path) => {
                self.writes.lock().unwrap().push(WrittenEntry {
                    path,
                    contents: data.to_vec(),
                });
                reply.written(data.len().try_into().unwrap());
            }
            Err(err) => reply.error(err),
        }
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        let entries = self.path(ino).and_then(|path| {
            let mut entries = Vec::new();
            for entry in fs::read_dir(self.source_path.join(&path))? {
                let entry = entry?;
                let kind = FileType::from_std(entry.file_type()?).ok_or(Errno::EIO)?;
                let name = entry.file_name();
                entries.push((self.inode(&path.join(&name)), kind, name));
            }
            Ok(entries)
        });

        match entries {
            Ok(entries) => {
                let skip = usize::try_from(offset).unwrap();
                for (i, (ino, kind, name)) in entries.into_iter().enumerate().skip(skip) {
                    if reply.add(ino, i as u64 + 1, kind, name) {
                        break;
                    }
                }
                reply.ok();
            }
            Err(err) => reply.error(err),
        }
    }

    fn statx(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _fh: Option<FileHandle>,
        _flags: u32,
        _mask: StatxMask,
        reply: ReplyStatx,
    ) {
        reply.error(Errno::ENOSYS);
    }

    fn flush(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _fh: FileHandle,
        _lock_owner: LockOwner,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    fn getxattr(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _name: &OsStr,
        _size: u32,
        reply: ReplyXattr,
    ) {
        reply.error(Errno::NO_XATTR);
    }
}
