// SPDX-License-Identifier: (MIT OR Apache-2.0)

use std::{
    io::{Read, Result, Seek, SeekFrom},
    sync::{Arc, RwLock},
};

use crate::BLOCK_SIZE;

/// A trait for objects which can be read by logical block addresses.
pub trait ISO9660Reader {
    /// Read the block(s) at a given LBA (logical block address)
    fn read_at(&mut self, buf: &mut [u8], lba: u64) -> Result<usize>;
}

impl<T: Read + Seek> ISO9660Reader for T {
    fn read_at(&mut self, buf: &mut [u8], lba: u64) -> Result<usize> {
        self.seek(SeekFrom::Start(lba * u64::from(BLOCK_SIZE)))?;
        self.read(buf)
    }
}

// TODO: Figure out if sane API possible without Rc/RefCell
pub(crate) struct FileRef<T: ISO9660Reader>(Arc<RwLock<T>>);

impl<T: ISO9660Reader> Clone for FileRef<T> {
    fn clone(&self) -> FileRef<T> {
        FileRef(self.0.clone())
    }
}

impl<T: ISO9660Reader> FileRef<T> {
    pub fn new(reader: T) -> FileRef<T> {
        FileRef(Arc::new(RwLock::new(reader)))
    }

    /// Read the block(s) at a given LBA (logical block address)
    pub fn read_at(&self, buf: &mut [u8], lba: u64) -> Result<usize> {
        (*self.0.as_ref()).write().unwrap().read_at(buf, lba)
    }
}
