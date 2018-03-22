use std::str::FromStr;
use std::io::{self, Read, Seek, SeekFrom, Write, Initializer};
use std::cmp::min;
use std::mem;
use std::fmt;

use time::Tm;

use super::DirectoryEntryHeader;
use ::{FileRef, Result, ISOError};

#[derive(Clone)]
pub struct ISOFile {
    pub(crate) header: DirectoryEntryHeader,
    pub identifier: String,
    // File version; ranges from 1 to 32767
    pub version: u16,
    file: FileRef,
    buf: [u8; 2048],
    buf_lba: Option<u64>,
    seek: u64
}

impl fmt::Debug for ISOFile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ISOFile")
           .field("header", &self.header)
           .field("version", &self.identifier)
           .field("file", &self.file)
           .field("seek", &self.seek)
           .finish()
    }
}

impl ISOFile {
    pub(crate) fn new(header: DirectoryEntryHeader, identifier: &str, file: FileRef) -> Result<ISOFile> {
        // Files (not directories) in ISO 9660 have a version number, which is
        // provided at the end of the identifier, seperated by ';'
        let error = ISOError::InvalidFs("File indentifier missing ';'");
        let idx = identifier.rfind(';').ok_or(error)?;

        let ver_str = &identifier[idx+1..];
        let mut name = &identifier[..idx];
        let version = u16::from_str(ver_str)?;

        // Files without an extension have a '.' at the end
        name = name.trim_right_matches('.');

        Ok(ISOFile {
            header,
            identifier: name.to_string(),
            version,
            file,
            buf: unsafe { mem::uninitialized() },
            buf_lba: None,
            seek: 0
        })
    }

    pub fn size(&self) -> u32 {
        self.header.extent_length
    }

    pub fn time(&self) -> Tm {
        self.header.time
    }
}

impl Read for ISOFile {
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }

    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let start_lba = self.header.extent_loc;

        let mut seek = self.seek;
        while !buf.is_empty() && seek < self.size() as u64 {
            let lba = start_lba as u64 + (seek / 2048);
            if self.buf_lba != Some(lba) {
                self.file.read_at(&mut self.buf, lba)?;
                self.buf_lba = Some(lba);
            }

            let start = seek as usize % 2048;
            let end = min(self.size() as usize - (seek as usize / 2048) * 2048, 2048);
            seek += buf.write(&self.buf[start..end]).unwrap() as u64;
        }

        let bytes = seek - self.seek;
        self.seek = seek;
        Ok(bytes as usize)
    }
}

impl Seek for ISOFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let seek = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => self.size() as i64 + pos,
            SeekFrom::Current(pos) => self.seek as i64 + pos,
        };

        if seek < 0 {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid seek"))
        } else { 
            self.seek = seek as u64;
            Ok(self.seek)
        }
    }
}
