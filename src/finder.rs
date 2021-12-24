use std::{
    fmt,
    io,
    num::NonZeroI8,
};

use derive_more::{From, Into, Display};

use four_cc::FourCC;

use bitvec::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderInfo {
    pub file_type: FileType,
    pub creator: Creator,
    pub flags: FinderFlags,
    pub location: Point,
    pub folder: Folder,
}

impl From<&[u8; 16]> for FinderInfo {
    fn from(bytes: &[u8; 16]) -> Self {
        let file_type = FileType::from(&bytes[..4]);
        let bytes = &bytes[4..];
        let creator = Creator::from(&bytes[..4]);
        let bytes = &bytes[4..];
        let flags = (&[bytes[0], bytes[1]]).into();
        // TODO: Parse the folder and location.
        Self {
            file_type,
            creator,
            flags,
            folder: Default::default(),
            location: Default::default(),
        }
    }
}

impl TryFrom<&[u8]> for FinderInfo {
    type Error = io::Error;
    fn try_from(bytes: &[u8]) -> io::Result<Self> {
        let bytes: Option<&[u8; 16]> = bytes.try_into().ok();
        if let Some(bytes) = bytes {
            Ok(bytes.into())
        } else {
            Err(io::ErrorKind::UnexpectedEof.into())
        }
    }
}

/// Mac File Type code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileType(FourCC);

impl From<&[u8; 4]> for FileType {
    fn from(buf: &[u8; 4]) -> Self {
        Self(buf.into())
    }
}

impl From<&[u8]> for FileType {
    fn from(buf: &[u8]) -> Self {
        Self(buf.into())
    }
}

/// Mac Creator code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Creator(FourCC);

impl From<&[u8; 4]> for Creator {
    fn from(buf: &[u8; 4]) -> Self {
        Self(buf.into())
    }
}

impl From<&[u8]> for Creator {
    fn from(buf: &[u8]) -> Self {
        Self(buf.into())
    }
}

/// Various flags that are either manipulated by the Finder or influence the way
/// the Finder will present the file.
#[derive(Default, Clone, Copy, PartialEq, Eq, From, Into, Display)]
#[display(fmt="{:#b}", _0)]
pub struct FinderFlags(u16);

impl FinderFlags {
    fn inner(&self) -> &BitSlice<Msb0, u16> {
        self.0.view_bits()
    }
    #[deprecated]
    pub fn is_on_desktop(&self) -> bool {
        self.inner()[0]
    }
    pub fn color(&self) -> u8 {
        self.inner()[1..4].load_be()
    }
    #[deprecated]
    pub fn color_reserved(&self) -> bool {
        self.inner()[4]
    }
    #[deprecated]
    pub fn requires_switch_launch(&self) -> bool {
        self.inner()[5]
    }
    pub fn is_shared(&self) -> bool {
        self.inner()[6]
    }
    pub fn has_no_inits(&self) -> bool {
        self.inner()[7]
    }
    pub fn has_been_inited(&self) -> bool {
        self.inner()[8]
    }
    pub fn has_custom_icon(&self) -> bool {
        self.inner()[10]
    }
    pub fn is_stationery(&self) -> bool {
        self.inner()[11]
    }
    pub fn name_locked(&self) -> bool {
        self.inner()[12]
    }
    pub fn has_bundle(&self) -> bool {
        self.inner()[13]
    }
    pub fn is_invisible(&self) -> bool {
        self.inner()[14]
    }
    pub fn is_alias(&self) -> bool {
        self.inner()[15]
    }
}

impl fmt::Debug for FinderFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FinderFlags({})", self)
    }
}

impl From<&[u8; 2]> for FinderFlags {
    fn from(bytes: &[u8; 2]) -> Self {
        Self::from(u16::from_be_bytes(*bytes))
    }
}

/// A 2-dimensional point in QuickDraw's coordinate system
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    vertical: i16,
    horizontal: i16,
}

/// The ID of the window representing the folder containing this file
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Folder(u16);

/// A bunch of extra information which is not very useful to the typical
/// developer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedFinderInfo {
    icon_id: i16,
    filename_script: FilenameScript,
    comment_id: i16,
    put_away_from: i32,
}

/// The script used to display the filename. If unspecified, then the finder
/// should use whatever the user currently is using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameScript {
    Unspecified,
    Script(NonZeroI8),
}

impl Default for FilenameScript {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl From<i8> for FilenameScript {
    fn from(value: i8) -> Self {
        if let Some(script) = NonZeroI8::new(value) {
            Self::Script(script)
        } else {
            Self::Unspecified
        }
    }
}

impl Into<i8> for FilenameScript {
    fn into(self) -> i8 {
        match self {
            Self::Unspecified => 0i8,
            Self::Script(value) => value.get(),
        }
    }
}
