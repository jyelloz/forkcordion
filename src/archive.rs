use derive_more::{From, Into, Display};

use super::{
    FinderInfo,
    MacInfo,
    Filename,
    Dates,
    Comment,
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
