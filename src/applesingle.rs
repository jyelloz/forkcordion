use std::{
    collections::BTreeMap,
    fmt,
    io::{
        self,
        prelude::*,
    },
};

use num_enum::{TryFromPrimitive, IntoPrimitive};
use deku::prelude::*;

use super::{
    Filename,
    Comment,
    Dates,
    FinderInfo,
    MacInfo,
    archive::Archive,
    io::{
        ReadExt as _,
        CountingReader,
    },
};

const MAGIC: u32 = 0x0005_1600;
const VERSION: u32 = 0x0002_0000;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub enum ArchiveMember<'a> {
    DataFork(Box<dyn 'a + Read>),
    ResourceFork(Box<dyn 'a + Read>),
    RealName(Filename),
    Comment(Comment),
    FileDates(Dates),
    FinderInfo(FinderInfo),
    MacInfo(MacInfo),
    Other(u32, Box<dyn 'a + Read>),
}

impl <'a> fmt::Debug for ArchiveMember<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataFork(_) => write!(f, "DataFork(..)"),
            Self::ResourceFork(_) => write!(f, "ResourceFork(..)"),
            Self::RealName(filename) => write!(f, "RealName({})", filename),
            Self::Comment(comment) =>  write!(f, "Comment({})", comment),
            Self::FileDates(dates) => write!(f, "FileDates({:?})", dates),
            Self::FinderInfo(info) => write!(f, "FinderInfo({:?})", info),
            Self::MacInfo(info) => write!(f, "MacInfo({})", info),
            Self::Other(id, _) => write!(f, "Other({}, ..)", id),
        }
    }
}

struct AppleSingleArchiveReader<R> {
    reader: CountingReader<R>,
    header: ArchiveHeader,
}

impl <R: Read> AppleSingleArchiveReader<R> {
    fn new(reader: R) -> io::Result<Self> {
        let mut archive = Self {
            reader: reader.counting(),
            header: ArchiveHeader::default(),
        };
        archive.read_magic()?;
        archive.read_header()?;
        Ok(archive)
    }
    fn read_magic(&mut self) -> io::Result<()> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        if u32::from_be_bytes(bytes) != MAGIC {
            let e: io::Result<_> = Err(io::ErrorKind::Other.into());
            e?;
        }
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        if u32::from_be_bytes(bytes) != VERSION {
            let e: io::Result<_> = Err(io::ErrorKind::Other.into());
            e?;
        }
        Ok(())
    }
    fn read_header(&mut self) -> io::Result<()> {
        let mut gap = [0u8; 16];
        self.read_exact(&mut gap)?;
        let mut bytes = [0u8; 2];
        self.read_exact(&mut bytes)?;
        let n_segments = u16::from_be_bytes(bytes);
        for _ in 0..n_segments {
            self.read_segment()?;
        }
        Ok(())
    }
    fn read_segment(&mut self) -> io::Result<()> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        let id = u32::from_be_bytes(bytes);
        self.read_exact(&mut bytes)?;
        let offset = u32::from_be_bytes(bytes);
        self.read_exact(&mut bytes)?;
        let len = u32::from_be_bytes(bytes);
        self.header.segments.insert(id, Segment { id, offset, len });
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

impl <R: Read> Read for AppleSingleArchiveReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
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
    fn wrap(self) -> io::Result<ArchiveMember<'a>> {
        let Self { segment, mut reader } = self;
        let len = segment.len_usize();
        let id = segment.id;
        let member = match segment.entry_type() {
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
            Some(EntryType::ResourceFork) => ArchiveMember::ResourceFork(
                Box::new(reader)
            ),
            Some(EntryType::DataFork) => ArchiveMember::DataFork(
                Box::new(reader)
            ),
            _ => ArchiveMember::Other(id, Box::new(reader)),
        };
        Ok(member)
    }
}

pub fn parse<R: Read>(archive: R) -> io::Result<Archive<R>> {
    let mut reader = AppleSingleArchiveReader::new(archive)?;
    let segments = reader.segments_by_offset();
    for segment in segments {
        let member = SegmentReader::from_segment(segment, &mut reader)
            .and_then(SegmentReader::wrap)?;
        eprintln!("{:?}", member);
        match member {
            ArchiveMember::ResourceFork(mut fork) => {
                eprintln!("writing rsrc fork {:?} to stdout", segment);
                io::copy(&mut fork, &mut io::stdout())?;
            },
            ArchiveMember::DataFork(mut fork) => {
                eprintln!("writing data fork {:?} to stdout", segment);
                io::copy(&mut fork, &mut io::stdout())?;
            },
            _ => {},
        };
    }
    Archive::builder()
        .format(FORMAT_NAME.into())
        .build()
        .ok_or(io::ErrorKind::Other.into())
}
