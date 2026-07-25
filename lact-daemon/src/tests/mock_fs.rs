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
        flush, fsync, lseek, read, release, access, getattr, listxattr, lookup, open, readdir,
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
