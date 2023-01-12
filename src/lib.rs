use std::{
    fmt,
    io::{Read, Seek, Write, Result, SeekFrom},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    id: u32,
    offset: u32,
    len: u32,
}

impl Entry {
    /// Applies the limit defined in this Entry to the given input stream. If
    /// your stream is seekable, you should use
    /// [`fixate()`][Entry::fixate] which will ensure that the resulting
    /// stream captures the exact region that the entry represents.
    pub fn limit<'a, R: Read + 'a>(&self, stream: R) -> Result<Box<dyn Read + 'a>> {
        Ok(Box::new(stream.take(self.len as u64)))
    }
    /// Applies the boundaries defined in this Entry to the given seekable
    /// input stream. This method will seek to the offset contained in this
    /// structure and restrict the amount of readable bytes in the returned
    /// value to the amount of bytes in the entry.
    pub fn fixate<'a, R: Read + Seek + 'a>(&self, mut stream: R) -> Result<Box<dyn Read + 'a>> {
        stream.seek(SeekFrom::Start(self.offset as u64))?;
        Ok(Box::new(stream.take(self.len as u64)))
    }
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
    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }
}

struct SectionWriter<'a, W> {
    archive: &'a mut ArchiveWriter<W>,
    entry: &'a Entry,
    position: usize,
}

impl <'a, W: Write + Seek> Write for SectionWriter<'a, W> {
    fn flush(&mut self) -> Result<()> {
        self.archive.flush()?;
        Ok(())
    }
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
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
