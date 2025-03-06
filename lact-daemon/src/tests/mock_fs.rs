use easy_fuser::{
    templates::{
        mirror_fs::{MirrorFsReadOnly, MirrorFsTrait},
        DefaultFuseHandler,
    },
    types::{
        BorrowedFileHandle, FUSESetXAttrFlags, FUSEWriteFlags, FileAttribute, FuseResult,
        OpenFlags, SetAttrRequest,
    },
    FuseHandler,
};
use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct MockSysfs {
    inner: MirrorFsReadOnly,
    pub writes: Arc<Mutex<Vec<(PathBuf, String)>>>,
}

impl MockSysfs {
    pub fn new(source_path: PathBuf) -> Self {
        MockSysfs {
            inner: MirrorFsReadOnly::new(source_path, DefaultFuseHandler::new()),
            writes: Arc::default(),
        }
    }
}

impl FuseHandler<PathBuf> for MockSysfs {
    fn get_inner(&self) -> &dyn FuseHandler<PathBuf> {
        &self.inner
    }

    fn write(
        &self,
        _req: &easy_fuser::prelude::RequestInfo,
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
        req: &easy_fuser::prelude::RequestInfo,
        file_id: PathBuf,
        _attrs: SetAttrRequest,
    ) -> FuseResult<FileAttribute> {
        self.getattr(req, file_id, None)
    }

    fn getxattr(
        &self,
        _req: &easy_fuser::prelude::RequestInfo,
        _file_id: PathBuf,
        _name: &OsStr,
        _size: u32,
    ) -> FuseResult<Vec<u8>> {
        Ok(vec![])
    }

    fn setxattr(
        &self,
        _req: &easy_fuser::prelude::RequestInfo,
        _file_id: PathBuf,
        _name: &OsStr,
        _value: Vec<u8>,
        _flags: FUSESetXAttrFlags,
        _position: u32,
    ) -> FuseResult<()> {
        Ok(())
    }
}
