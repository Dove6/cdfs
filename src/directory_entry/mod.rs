pub use self::isodirectory::ISODirectory;
pub use self::isofile::ISOFile;

use crate::parse::{DirectoryEntryHeader, FileFlags};
use crate::{ISO9660Reader, Result, FileRef};

mod isodirectory;
mod isofile;

#[derive(Clone, Debug)]
pub enum DirectoryEntry<T: ISO9660Reader> {
    Directory(ISODirectory<T>),
    File(ISOFile<T>)
}

impl<T: ISO9660Reader> DirectoryEntry<T> {
    pub(crate) fn new(header: DirectoryEntryHeader, identifier: String,
                      file: FileRef<T>) -> Result<Self> {
        if header.file_flags.contains(FileFlags::DIRECTORY) {
            Ok(DirectoryEntry::Directory(ISODirectory::new(
                header,
                identifier,
                file
            )))
        } else {
            Ok(DirectoryEntry::File(ISOFile::new(
                header,
                identifier,
                file
            )?))
        }
    }

    pub(crate) fn header(&self) -> &DirectoryEntryHeader {
        match *self {
            DirectoryEntry::Directory(ref dir) => &dir.header,
            DirectoryEntry::File(ref file) => &file.header,
        }

    }

    pub fn identifier(&self) -> &str {
        match *self {
            DirectoryEntry::Directory(ref dir) => &dir.identifier,
            DirectoryEntry::File(ref file) => &file.identifier,
        }
    }
}
