use std::{mem, str};

use time::Tm;

use ::{DirectoryEntry, ISOFile, FileRef, Result, ISOError};
use ::parse::{DirectoryEntryHeader, FileFlags};

// Like try!, but wrap in Some()
macro_rules! try_some {
    ( $res:expr ) => {
        match $res {
            Ok(val) => val,
            Err(err) => {
                return Some(Err(err.into()));
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct ISODirectory {
    pub(crate) header: DirectoryEntryHeader,
    pub identifier: String,
    file: FileRef
}

impl ISODirectory {
    pub(crate) fn new(header: DirectoryEntryHeader, mut identifier: String, file: FileRef) -> ISODirectory {
        if &identifier == "\u{0}" {
            identifier = ".".to_string();
        } else if &identifier == "\u{1}" {
            identifier = "..".to_string();
        }

        ISODirectory {
            header,
            identifier,
            file
        }
    }

    pub fn block_count(&self) -> u32 {
        let len = self.header.extent_length;
        (len + 2048 - 1) / 2048 // ceil(len / 2048)
    }

    pub fn contents(&self) -> ISODirectoryIterator {
        ISODirectoryIterator {
            loc: self.header.extent_loc,
            block_count: self.block_count(),
            file: self.file.clone(),
            block: unsafe { mem::uninitialized() },
            block_num: 0,
            block_pos: 0,
            have_block: false
        }
    }

    pub fn time(&self) -> Tm {
        self.header.time
    }

    pub fn find(&self, identifier: &str) -> Result<Option<DirectoryEntry>> {
        for entry in self.contents() {
            let entry = entry?;
            if entry.header().file_flags.contains(FileFlags::ASSOCIATEDFILE) {
                continue;
            }
            if entry.identifier().eq_ignore_ascii_case(identifier) {
                return Ok(Some(entry));
            }
        }

        Ok(None)
    }
}

pub struct ISODirectoryIterator {
    loc: u32,
    block_count: u32,
    file: FileRef,
    block: [u8; 2048],
    block_num: u32,
    block_pos: usize,
    have_block: bool
}

impl Iterator for ISODirectoryIterator {
    type Item = Result<DirectoryEntry>;

    fn next(&mut self) -> Option<Result<DirectoryEntry>> {
        if self.block_num == self.block_count {
            return None;
        }

        // If we have reached the end of one block, read another
        if !self.have_block ||
           self.block_pos >= (2048 - 33) ||
           // All bytes after the last directory entry are zero.
           self.block[self.block_pos] == 0 {

            if self.have_block {
                self.block_num += 1;
            }
            self.block_pos = 0;
            self.have_block = true;

            if self.block_num == self.block_count {
                return None;
            }

            let count = try_some!(self.file.read_at(
                    &mut self.block,
                    self.loc as u64 + self.block_num as u64));

            if count != 2048 {
                return Some(Err(ISOError::ReadSize(2048, count)));
            }
         }

        let (header, identifier) = try_some!(
            DirectoryEntryHeader::parse(&self.block[self.block_pos..]));
        self.block_pos += header.length as usize;

        let entry = if header.file_flags.contains(FileFlags::DIRECTORY) {
            DirectoryEntry::Directory(ISODirectory::new(
                header,
                identifier,
                self.file.clone()
            ))
        } else {
            DirectoryEntry::File(try_some!(ISOFile::new(
                header,
                identifier,
                self.file.clone()
            )))
        };

        Some(Ok(entry))
    }
}
