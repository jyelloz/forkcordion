use std::{
    collections::BTreeMap,
    io::{
        self,
        prelude::*,
    },
};

use num_enum::{TryFromPrimitive, IntoPrimitive};

use super::{
    archive::Archive,
    DataFork,
    ResourceFork,
    Filename,
    Comment,
    Dates,
    FinderInfo,
    MacInfo,
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

#[derive(Debug)]
pub enum ArchiveMember {
    DataFork(DataFork),
    ResourceFork(ResourceFork),
    RealName(Filename),
    Comment(Comment),
    FileDates(Dates),
    FinderInfo(FinderInfo),
    MacInfo(MacInfo),
    Other(Segment),
}

struct AppleSingleArchiveReader<R: Read> {
    reader: R,
    header: ArchiveHeader,
    position: usize,
}

impl <R: Read> AppleSingleArchiveReader<R> {
    fn new(reader: R) -> io::Result<Self> {
        let mut archive = Self {
            reader,
            header: ArchiveHeader::default(),
            position: 0,
        };
        archive.read_magic()?;
        archive.read_header()?;
        Ok(archive)
    }
    fn read_magic(&mut self) -> io::Result<()> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        if u32::from_be_bytes(bytes) != MAGIC {
            eprintln!("invalid magic: {:?}", &bytes);
            let e: Result<Self, io::Error> = Err(io::ErrorKind::Other.into());
            e?;
        }
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        if u32::from_be_bytes(bytes) != VERSION {
            eprintln!("invalid version: {:?}", &bytes);
            let e: Result<Self, io::Error> = Err(io::ErrorKind::Other.into());
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
    fn segments(self) -> SegmentIterator<R> {
        let segments = self.header.segments_by_offset();
        SegmentIterator {
            reader: self,
            segments: Box::new(segments.into_iter()),
        }
    }
}

impl <R: Read> Read for AppleSingleArchiveReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.reader.read(buf)?;
        self.position += bytes;
        Ok(bytes)
    }
}

struct SegmentIterator<R: Read> {
    reader: AppleSingleArchiveReader<R>,
    segments: Box<dyn std::iter::Iterator<Item=Segment>>,
}

impl <R: Read> SegmentIterator<R> {
    fn skip_to(&mut self, segment: Segment) -> io::Result<()> {
        let Self { reader, .. } = self;
        let offset = segment.offset as usize;
        let position = reader.position;
        if position > offset {
            Err(io::ErrorKind::Unsupported)?;
        }
        let diff = (offset - position) as u64;
        let mut take = reader.take(diff);
        io::copy(&mut take, &mut io::sink())?;
        Ok(())
    }
    fn wrap_segment(&mut self, segment: Segment) -> Option<ArchiveMember> {
        eprintln!("wrapping segment {:?}", &segment);
        self.skip_to(segment).expect("failed to seek");
        let Self { reader, ..} = self;
        let len = segment.len as usize;
        match EntryType::try_from(segment.id) {
            Err(_) => Some(ArchiveMember::Other(segment)),
            Ok(EntryType::RealName) => {
                let mut buf = Vec::with_capacity(len);
                buf.resize(len, 0);
                reader.read_exact(&mut buf).expect("failed to read filename");
                Some(ArchiveMember::RealName(Filename(buf)))
            },
            Ok(EntryType::FinderInfo) => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf).expect("failed to read finder info");
                let finf = FinderInfo::from(&buf);
                Some(ArchiveMember::FinderInfo(finf))
            },
            Ok(EntryType::Comment) => {
                let mut buf = Vec::with_capacity(len);
                let len = reader.take(len as u64)
                    .read_to_end(&mut buf)
                    .expect("failed to read comment");
                buf.truncate(len);
                Some(ArchiveMember::Comment(Comment(buf)))
            },
            Ok(id) => {
                eprintln!(
                    "unsupported archive member type: {:?}, {:?}",
                    id,
                    segment,
                );
                None
            }
        }
    }
}

impl <R: Read> Iterator for SegmentIterator<R> {
    type Item = ArchiveMember;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let segment = self.segments.next()?;
            if let Some(member) = self.wrap_segment(segment) {
                return Some(member);
            }
        }
    }
}

pub fn parse<R: Read>(archive: R) -> io::Result<Archive> {
    let reader = AppleSingleArchiveReader::new(archive)?;
    let iter = reader.segments();
    for item in iter {
        println!("{:?}", &item);
    }
    Archive::builder()
        .format(FORMAT_NAME.into())
        .build()
        .ok_or(io::ErrorKind::Other.into())
}
