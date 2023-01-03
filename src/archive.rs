use std::{
    io::Read,
    marker::PhantomData,
};

use derive_more::{From, Into, Display};

use super::{
    DataFork,
    ResourceFork,
    FinderInfo,
    MacInfo,
    Filename,
    Dates,
    Comment,
};

#[derive(Debug, Clone, Copy, From, Into, Display)]
#[display(fmt = "{}", _0)]
pub struct Format(&'static str);

pub struct ArchiveBuilder<R> {
    format: Option<Format>,
    data: Option<DataFork<R>>,
    rsrc: Option<ResourceFork<R>>,
    finf: Option<FinderInfo>,
    minf: Option<MacInfo>,
    name: Option<Filename>,
    date: Option<Dates>,
    comment: Option<Comment>,
}

impl <R: Read> ArchiveBuilder<R> {
    pub fn new() -> Self {
        Self {
            format: None,
            data: None,
            rsrc: None,
            finf: None,
            minf: None,
            name: None,
            date: None,
            comment: None,
        }
    }
    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }
    pub fn build(self) -> Option<Archive<R>> {
        let archive = Archive {
            format: self.format?,
            finf: self.finf,
            _phantom: PhantomData,
        };
        Some(archive)
    }
}

pub struct Archive<R> {
    format: Format,
    finf: Option<FinderInfo>,
    _phantom: PhantomData<R>,
}

impl <R: Read> Archive<R> {
    pub fn builder() -> ArchiveBuilder<R> {
        ArchiveBuilder::new()
    }
    pub fn data_fork(&self) -> Option<DataFork<R>> {
        None
    }
    pub fn rsrc_fork(&self) -> Option<ResourceFork<R>> {
        None
    }
    pub fn finder_info(&self) -> Option<FinderInfo> {
        self.finf
    }
    pub fn format(&self) -> Format {
        self.format
    }
}
