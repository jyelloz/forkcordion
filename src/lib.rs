use std::{
    fmt,
    io::{Read, Seek, Write, Result as IOResult},
};

pub(crate) mod io;
mod finder;
mod archive;
mod date;
pub mod applesingle;

pub use crate::date::{Date, Dates};
pub use crate::finder::{
    FileType,
    Creator,
    FinderFlags,
    FinderInfo,
    MacInfo,
};

pub struct Header {
    entries: Vec<Entry>,
}

impl Header {
    fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
}

pub struct EntryBuilder {
    id: Option<u32>,
    offset: Option<u32>,
    len: Option<u32>,
}

pub struct Entry {
    id: u32,
    offset: u32,
    len: u32,
}

/// The raw data fork of a file.
pub struct DataFork<R>(R);

impl <R> fmt::Debug for DataFork<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DataFork").finish()
    }
}

/// The raw resource fork of a file.
pub struct ResourceFork<R>(pub R);

impl <R> fmt::Debug for ResourceFork<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResourceFork").finish()
    }
}

pub trait ForkExt<'r, R: Read> {
    /// Assume this object is a Data Fork
    fn data_fork(self) -> DataFork<R>;
    /// Assume this object is a Resource Fork
    fn rsrc_fork(self) -> ResourceFork<R>;
}

impl <'r, R: Read> ForkExt<'r, R> for R {
    fn data_fork(self) -> DataFork<R> {
        DataFork(self)
    }
    fn rsrc_fork(self) -> ResourceFork<R> {
        ResourceFork(self)
    }
}

/// The "Real Name" as mentioned in the AppleSingle specification.
#[derive(Clone)]
pub struct Filename(Vec<u8>);

impl fmt::Debug for Filename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Filename({})", self)
    }
}
impl fmt::Display for Filename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => write!(f, "{:?}", &self.0),
        }
    }
}

/// The comment visible in the Mac Finder's Get Info window.
#[derive(Clone)]
pub struct Comment(Vec<u8>);

impl fmt::Debug for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Comment({})", self)
    }
}
impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => write!(f, "{:?}", &self.0),
        }
    }
}

pub struct ArchiveWriter<W> {
    header: Header,
    file: W,
}

impl <W: Write> Write for ArchiveWriter<W> {
    fn flush(&mut self) -> IOResult<()> {
        self.file.flush()
    }
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.file.write(buf)
    }
}

struct SectionWriter<'a, W> {
    archive: &'a mut ArchiveWriter<W>,
    entry: &'a Entry,
    position: usize,
}

impl <'a, W: Write + Seek> Write for SectionWriter<'a, W> {
    fn flush(&mut self) -> IOResult<()> {
        self.archive.flush()?;
        Ok(())
    }
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        let Self { position, entry, archive } = self;
        let len = entry.len as usize;
        if *position >= len {
            return Ok(0);
        }
        let budget = buf.len().min(len - *position);
        let buf = &buf[..budget];
        let progress = archive.write(buf)?;
        *position += progress;
        Ok(progress)
    }
}
