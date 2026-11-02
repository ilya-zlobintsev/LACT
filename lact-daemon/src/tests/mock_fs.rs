use easy_fuser::{
    delegate_fs,
    fuse_parallel::prelude::*,
    fuse_presets::{
        DefaultFuseHandler,
        mirror_fs::{MirrorFsReadOnly, MirrorFsTrait},
    },
};
use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Files which are writeable as-is (reads will return last write's content)
const WRITEABLE_FILES: &[&str] = &["power_dpm_force_performance_level"];

pub struct MockSysfs {
    mirror_fs: MirrorFsReadOnly,
    default_fs: DefaultFuseHandler<PathBuf>,
    pub writes: Arc<Mutex<Vec<(PathBuf, String)>>>,
}

impl MockSysfs {
    pub fn new(source_path: PathBuf) -> Self {
        MockSysfs {
            mirror_fs: MirrorFsReadOnly::new(source_path),
            default_fs: DefaultFuseHandler::new(),
            writes: Arc::default(),
        }
    }
}

impl FuseHandler for MockSysfs {
    type TId = PathBuf;

    delegate_fs! { mirror_fs, [
        flush, fsync, lseek, release, access, getattr, listxattr, lookup, open, readdir,
        readlink
    ] }

    delegate_fs! { default_fs, [
        copy_file_range, fallocate, create, mkdir, mknod, removexattr, rename, rmdir, symlink,
        unlink, bmap, forget, fsyncdir, getlk, ioctl, link, opendir, releasedir, setlk, statfs
    ] }

    fn write(
        &self,
        _req: &RequestInfo,
        file_id: PathBuf,
        _file_handle: BorrowedFileHandle,
        _seek: std::io::SeekFrom,
        data: Vec<u8>,
        _write_flags: FUSEWriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<u64>,
    ) -> FuseResult<u32> {
        self.writes
            .lock()
            .unwrap()
            .push((file_id, String::from_utf8_lossy(&data).into_owned()));

        Ok(data.len().try_into().unwrap())
    }

    fn read(
        &self,
        req: &RequestInfo,
        file_id: Self::TId,
        file_handle: BorrowedFileHandle<'_>,
        seek: SeekFrom,
        size: u32,
        flags: FUSEOpenFlags,
        lock_owner: Option<u64>,
    ) -> FuseResult<Vec<u8>> {
        let file_name = file_id.file_name().unwrap().to_str().unwrap();
        if WRITEABLE_FILES.contains(&file_name) {
            let writes = self.writes.lock().unwrap();
            if let Some((_, written)) = writes.iter().rfind(|(id, _)| *id == file_id) {
                return Ok(written.as_bytes().to_vec());
            }
        }

        self.mirror_fs
            .read(req, file_id, file_handle, seek, size, flags, lock_owner)
    }

    fn setattr(
        &self,
        req: &RequestInfo,
        file_id: PathBuf,
        _attrs: SetAttrRequest,
    ) -> FuseResult<FileAttribute> {
        self.getattr(req, file_id, None)
    }

    fn getxattr(
        &self,
        _req: &RequestInfo,
        _file_id: PathBuf,
        _name: &OsStr,
        _size: u32,
    ) -> FuseResult<Vec<u8>> {
        Ok(vec![])
    }

    fn setxattr(
        &self,
        _req: &RequestInfo,
        _file_id: PathBuf,
        _name: &OsStr,
        _value: Vec<u8>,
        _flags: FUSESetXAttrFlags,
        _position: u32,
    ) -> FuseResult<()> {
        Ok(())
    }
}
