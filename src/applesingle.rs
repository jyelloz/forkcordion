use std::{
    collections::BTreeMap,
    fmt,
    io::{
        self,
        Seek,
        SeekFrom,
        prelude::*,
    },
};

use num_enum::{TryFromPrimitive, IntoPrimitive};
use deku::prelude::*;

use super::{
    Comment,
    Dates,
    Entry,
    Filename,
    FinderInfo,
    MacInfo,
    archive::{
        Archive,
        SeekableArchive,
    },
    io::{
        ReadExt as _,
        CountingReader,
    },
};

const FORMAT_NAME: &str = "AppleSingle";

#[derive(
    Debug,
    PartialEq, Eq,
    PartialOrd, Ord,
    TryFromPrimitive, IntoPrimitive,
)]
#[repr(u32)]
enum EntryType {
    DataFork = 1,
    ResourceFork,
    RealName,
    Comment,
    IconBW,
    IconColor,
    FileDates = 8,
    FinderInfo,
    MacintoshFileInfo,
    ProDOSFileInfo,
    MSDOSFileInfo,
    AFPShortName,
    AFPFileInfo,
    AFPDirectoryID,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq)]
#[deku(endian = "big", magic = b"\x00\x05\x16\x00\x00\x02\x00\x00")]
struct AppleSingleHeader {
    #[deku(pad_bytes_before = "16")]
    n_segments: u16,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq)]
#[deku(endian = "big")]
pub struct Segment {
    pub id: u32,
    pub offset: u32,
    pub len: u32,
}

impl Segment {
    fn entry_type(&self) -> Option<EntryType> {
        self.id.try_into().ok()
    }
    pub fn offset_u64(&self) -> u64 {
        self.offset as u64
    }
    pub fn len_usize(&self) -> usize {
        self.len as usize
    }
    pub fn len_u64(&self) -> u64 {
        self.len as u64
    }
    fn wrap<R: Read>(&self, reader: &mut R) -> io::Result<ArchiveMember> {
        let len = self.len_usize();
        let entry: Entry = (*self).into();
        let member = match self.entry_type() {
            Some(EntryType::RealName) => {
                let mut buf = Vec::with_capacity(len);
                reader.read_to_end(&mut buf)?;
                ArchiveMember::RealName(Filename(buf))
            },
            Some(EntryType::Comment) => {
                let mut buf = Vec::with_capacity(len);
                reader.read_to_end(&mut buf)?;
                ArchiveMember::Comment(Comment(buf))
            },
            Some(EntryType::FinderInfo) => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf)?;
                let (_, info) = FinderInfo::from_bytes((&buf, 0))?;
                ArchiveMember::FinderInfo(info)
            },
            Some(EntryType::FileDates) => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf)?;
                let (_, dates) = Dates::from_bytes((&buf, 0))?;
                ArchiveMember::FileDates(dates)
            },
            Some(EntryType::MacintoshFileInfo) => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                let (_, info) = MacInfo::from_bytes((&buf, 0))?;
                ArchiveMember::MacInfo(info)
            },
            Some(EntryType::ResourceFork) => ArchiveMember::ResourceFork(entry),
            Some(EntryType::DataFork) => ArchiveMember::DataFork(entry),
            _ => ArchiveMember::Other(entry),
        };
        Ok(member)
    }
}

impl Into<Entry> for Segment {
    fn into(self) -> Entry {
        let Self { id, offset, len } = self;
        Entry { id, offset, len }
    }
}

#[derive(Default)]
struct ArchiveHeader {
    segments: BTreeMap<u32, Segment>,
}

impl ArchiveHeader {
    fn segments_by_offset(&self) -> Vec<Segment> {
        let mut segments: Vec<Segment> = self.segments.values()
            .cloned()
            .collect();
        segments.sort_by_key(|s| s.offset);
        segments
    }
}

pub enum ArchiveMember {
    DataFork(Entry),
    ResourceFork(Entry),
    RealName(Filename),
    Comment(Comment),
    FileDates(Dates),
    FinderInfo(FinderInfo),
    MacInfo(MacInfo),
    Other(Entry),
}

impl fmt::Debug for ArchiveMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataFork(entry) => write!(f, "DataFork({entry:?})"),
            Self::ResourceFork(entry) => write!(f, "ResourceFork({entry:?})"),
            Self::RealName(filename) => write!(f, "RealName({filename})"),
            Self::Comment(comment) =>  write!(f, "Comment({comment})"),
            Self::FileDates(dates) => write!(f, "FileDates({dates:?})"),
            Self::FinderInfo(info) => write!(f, "FinderInfo({info:?})"),
            Self::MacInfo(info) => write!(f, "MacInfo({})", info),
            Self::Other(entry) => write!(f, "Other({entry:?})"),
        }
    }
}

struct AppleSingleArchiveReader<R> {
    reader: CountingReader<R>,
    header: ArchiveHeader,
}

impl <R: Read> AppleSingleArchiveReader<R> {
    fn streaming(reader: R) -> io::Result<Self> {
        let mut archive = Self {
            reader: reader.counting(),
            header: ArchiveHeader::default(),
        };
        archive.read_header()?;
        Ok(archive)
    }
    fn read_header(&mut self) -> io::Result<()> {
        let mut bytes = [0u8; 26];
        self.read_exact(&mut bytes)?;
        let (_, header) = AppleSingleHeader::from_bytes((&bytes, 0))?;
        let AppleSingleHeader { n_segments } = header;
        for _ in 0..n_segments {
            self.read_segment()?;
        }
        Ok(())
    }
    fn read_segment(&mut self) -> io::Result<()> {
        let mut bytes = [0u8; 12];
        self.read_exact(&mut bytes)?;
        let (_, segment) = Segment::from_bytes((&bytes, 0))?;
        self.header.segments.insert(segment.id, segment);
        Ok(())
    }
    fn segments_by_offset(&self) -> Vec<Segment> {
        self.header.segments_by_offset()
    }
    pub fn skip_to(&mut self, offset: u64) -> io::Result<()> {
        self.reader.skip_to(offset)?;
        Ok(())
    }
}

impl <R: Read + Seek> AppleSingleArchiveReader<R> {
    fn seekable(reader: R) -> io::Result<Self> {
        let mut archive = Self {
            reader: reader.counting(),
            header: ArchiveHeader::default(),
        };
        archive.read_header()?;
        Ok(archive)
    }
}

impl <R: Read> Read for AppleSingleArchiveReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl <S: Seek> Seek for AppleSingleArchiveReader<S> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}

struct SegmentReader<'a, R: Read> {
    segment: Segment,
    reader: io::Take<&'a mut AppleSingleArchiveReader<R>>,
}

impl <'a, R: Read> SegmentReader<'a, R> {
    fn from_segment(segment: Segment, reader: &'a mut AppleSingleArchiveReader<R>) -> io::Result<Self> {
        reader.skip_to(segment.offset_u64())?;
        let reader = reader.take(segment.len_u64());
        Ok(Self { segment, reader })
    }
    fn wrap(self) -> io::Result<ArchiveMember> {
        let Self { segment, mut reader } = self;
        segment.wrap(&mut reader)
    }
}

pub enum Fork {
    Data,
    Rsrc,
    Other(u32),
}
pub trait Handler {
    fn sink<'a>(&'a mut self, fork: Fork) -> Option<Box<dyn Write + 'a>>;
}

pub fn parse<R: Read, H: Handler>(
    archive: R,
    handler: &mut H,
) -> io::Result<Archive> {
    let mut reader = AppleSingleArchiveReader::streaming(archive)?;
    let segments = reader.segments_by_offset();
    let mut builder = Archive::builder();
    builder.format(FORMAT_NAME.into());
    for segment in segments {
        let member = SegmentReader::from_segment(segment, &mut reader)
            .and_then(SegmentReader::wrap)?;
        match member {
            ArchiveMember::ResourceFork(segment) => {
                if let Some(mut sink) = handler.sink(Fork::Rsrc) {
                    let mut fork = Entry::from(segment)
                        .limit(&mut reader)?;
                    io::copy(&mut fork, &mut sink)?;
                }
            },
            ArchiveMember::DataFork(segment) => {
                if let Some(mut sink) = handler.sink(Fork::Data) {
                    let mut fork = Entry::from(segment)
                        .limit(&mut reader)?;
                    io::copy(&mut fork, &mut sink)?;
                }
            },
            ArchiveMember::Other(segment) => {
                if let Some(mut sink) = handler.sink(Fork::Other(segment.id)) {
                    let mut fork = Entry::from(segment)
                        .limit(&mut reader)?;
                    io::copy(&mut fork, &mut sink)?;
                }
            },
            ArchiveMember::RealName(name) => {
                builder.name(name);
            }
            ArchiveMember::Comment(comment) => {
                builder.comment(comment);
            }
            ArchiveMember::FinderInfo(finf) => {
                builder.finf(finf);
            }
            ArchiveMember::MacInfo(minf) => {
                builder.minf(minf);
            }
            ArchiveMember::FileDates(date) => {
                builder.date(date);
            }
        }
    }

    builder.build()
        .ok_or(io::ErrorKind::Other.into())
}

pub fn parse_seekable<R: Read + Seek>(
    mut archive: R,
) -> io::Result<SeekableArchive<R>> {
    let segments = AppleSingleArchiveReader::seekable(&mut archive)?
        .segments_by_offset();
    let mut builder = SeekableArchive::builder(archive);
    builder.format(FORMAT_NAME.into());
    for segment in segments {
        let member = {
            let mut reader = builder.entry(segment.into())?;
            segment.wrap(&mut reader)?
        };
        match member {
            ArchiveMember::ResourceFork(entry) => {
                builder.rsrc_fork(entry);
            },
            ArchiveMember::DataFork(entry) => {
                builder.data_fork(entry);
            },
            ArchiveMember::Other(_) => {
                // TODO: expose non-resource/data forks somehow
            },
            ArchiveMember::RealName(name) => {
                builder.name(name);
            }
            ArchiveMember::Comment(comment) => {
                builder.comment(comment);
            }
            ArchiveMember::FinderInfo(finf) => {
                builder.finf(finf);
            }
            ArchiveMember::MacInfo(minf) => {
                builder.minf(minf);
            }
            ArchiveMember::FileDates(date) => {
                builder.date(date);
            }
        };
    }
    builder.build()
        .ok_or(io::ErrorKind::Other.into())
}
