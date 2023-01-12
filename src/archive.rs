use std::io::{Read, Seek, Result};
use derive_more::{From, Into, Display};

use super::{
    FinderInfo,
    MacInfo,
    Filename,
    Dates,
    Comment,
    Entry,
};

#[derive(Debug, Clone, Copy, From, Into, Display)]
#[display(fmt = "{}", _0)]
pub struct Format(&'static str);

pub struct ArchiveBuilder {
    format: Option<Format>,
    finf: Option<FinderInfo>,
    minf: Option<MacInfo>,
    name: Option<Filename>,
    date: Option<Dates>,
    comment: Option<Comment>,
}

impl ArchiveBuilder {
    pub fn new() -> Self {
        Self {
            format: None,
            finf: None,
            minf: None,
            name: None,
            date: None,
            comment: None,
        }
    }
    pub fn format(&mut self, format: Format) -> &Self {
        self.format = Some(format);
        self
    }
    pub fn finf(&mut self, finf: FinderInfo) -> &Self {
        self.finf = Some(finf);
        self
    }
    pub fn minf(&mut self, minf: MacInfo) -> &Self {
        self.minf = Some(minf);
        self
    }
    pub fn date(&mut self, date: Dates) -> &Self {
        self.date = Some(date);
        self
    }
    pub fn name(&mut self, name: Filename) -> &Self {
        self.name = Some(name);
        self
    }
    pub fn comment(&mut self, comment: Comment) -> &Self {
        self.comment = Some(comment);
        self
    }
    pub fn build(&self) -> Option<Archive> {
        let archive = Archive {
            format: self.format?,
            finf: self.finf,
            minf: self.minf,
            date: self.date,
            name: self.name.clone(),
            comment: self.comment.clone(),
        };
        Some(archive)
    }
}

#[derive(Debug)]
pub struct Archive {
    format: Format,
    finf: Option<FinderInfo>,
    minf: Option<MacInfo>,
    date: Option<Dates>,
    name: Option<Filename>,
    comment: Option<Comment>,
}

impl Archive {
    pub fn builder() -> ArchiveBuilder {
        ArchiveBuilder::new()
    }
    pub fn finder_info(&self) -> Option<FinderInfo> {
        self.finf
    }
    pub fn mac_info(&self) -> Option<MacInfo> {
        self.minf
    }
    pub fn dates(&self) -> Option<Dates> {
        self.date
    }
    pub fn name(&self) -> Option<Filename> {
        self.name.clone()
    }
    pub fn comment(&self) -> Option<Comment> {
        self.comment.clone()
    }
    pub fn format(&self) -> Format {
        self.format
    }
}

pub struct SeekableArchiveBuilder<R> {
    archive: ArchiveBuilder,
    rsrc_fork: Option<Entry>,
    data_fork: Option<Entry>,
    file: R,
}

impl <R: Read + Seek> SeekableArchiveBuilder<R> {
    pub fn new(file: R) -> Self {
        Self {
            file,
            archive: ArchiveBuilder::new(),
            rsrc_fork: None,
            data_fork: None,
        }
    }
    pub fn entry<'a>(&'a mut self, entry: Entry) -> Result<Box<dyn Read + 'a>> {
        entry.fixate(&mut self.file)
    }
    pub fn format(&mut self, format: Format) -> &Self {
        self.archive.format(format);
        self
    }
    pub fn finf(&mut self, finf: FinderInfo) -> &Self {
        self.archive.finf(finf);
        self
    }
    pub fn minf(&mut self, minf: MacInfo) -> &Self {
        self.archive.minf(minf);
        self
    }
    pub fn date(&mut self, date: Dates) -> &Self {
        self.archive.date(date);
        self
    }
    pub fn name(&mut self, name: Filename) -> &Self {
        self.archive.name(name);
        self
    }
    pub fn comment(&mut self, comment: Comment) -> &Self {
        self.archive.comment(comment);
        self
    }
    pub fn data_fork(&mut self, data: Entry) -> &Self {
        self.data_fork = Some(data);
        self
    }
    pub fn rsrc_fork(&mut self, rsrc: Entry) -> &Self {
        self.rsrc_fork = Some(rsrc);
        self
    }
    pub fn build(self) -> Option<SeekableArchive<R>> {
        let archive = self.archive.build()?;
        let archive = SeekableArchive {
            format: archive.format,
            finf: archive.finf,
            minf: archive.minf,
            date: archive.date,
            name: archive.name,
            comment: archive.comment,
            file: self.file,
            rsrc_fork: self.rsrc_fork,
            data_fork: self.data_fork,
        };
        Some(archive)
    }
}

#[derive(Debug)]
pub struct SeekableArchive<R> {
    format: Format,
    finf: Option<FinderInfo>,
    minf: Option<MacInfo>,
    date: Option<Dates>,
    name: Option<Filename>,
    comment: Option<Comment>,
    rsrc_fork: Option<Entry>,
    data_fork: Option<Entry>,
    file: R,
}

impl <R: Read + Seek> SeekableArchive<R> {
    pub fn builder(file: R) -> SeekableArchiveBuilder<R> {
        SeekableArchiveBuilder::new(file)
    }
    pub fn finder_info(&self) -> Option<FinderInfo> {
        self.finf
    }
    pub fn mac_info(&self) -> Option<MacInfo> {
        self.minf
    }
    pub fn dates(&self) -> Option<Dates> {
        self.date
    }
    pub fn name(&self) -> Option<Filename> {
        self.name.clone()
    }
    pub fn comment(&self) -> Option<Comment> {
        self.comment.clone()
    }
    pub fn format(&self) -> Format {
        self.format
    }
    pub fn data_fork<'a>(&'a mut self) -> Result<Option<Box<dyn Read + 'a>>> {
        if let Some(entry) = self.data_fork {
            let reader = entry.fixate(&mut self.file)?;
            Ok(Some(reader))
        } else {
            Ok(None)
        }
    }
    pub fn rsrc_fork<'a>(&'a mut self) -> Result<Option<Box<dyn Read + 'a>>> {
        if let Some(entry) = self.rsrc_fork {
            let reader = entry.fixate(&mut self.file)?;
            Ok(Some(reader))
        } else {
            Ok(None)
        }
    }
}
