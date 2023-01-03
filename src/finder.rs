use std::{
    fmt,
    num::NonZeroI8,
};

use derive_more::{From, Into};

use four_cc::FourCC;

use bitvec::prelude::*;
use arrayref::array_ref;

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
        let file_type = FileType::from(array_ref![bytes, 0, 4]);
        let creator = Creator::from(array_ref![bytes, 4, 4]);
        let flags = FinderFlags::from(
            u16::from_be_bytes(*array_ref![bytes, 8, 2])
        );
        let location = Point::from(array_ref![bytes, 10, 4]);
        let folder = Folder::from(array_ref![bytes, 14, 2]);
        Self {
            file_type,
            creator,
            flags,
            location,
            folder,
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

/// Mac Creator code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Creator(FourCC);

impl From<&[u8; 4]> for Creator {
    fn from(buf: &[u8; 4]) -> Self {
        Self(buf.into())
    }
}

/// Various flags that are either manipulated by the Finder or influence the way
/// the Finder will present the file.
#[derive(Default, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct FinderFlags(u16);

impl FinderFlags {
    fn inner(&self) -> &BitSlice<u16, Lsb0> {
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

impl fmt::Display for FinderFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = vec![];
        #[allow(deprecated)]
        if self.is_on_desktop() {
            text.push("ON_DESKTOP".to_string());
        }
        text.push(format!("COLOR={}", self.color()));
        #[allow(deprecated)]
        if self.color_reserved() {
            text.push("COLOR_RESERVED".to_string());
        }
        #[allow(deprecated)]
        if self.requires_switch_launch() {
            text.push("REQUIRES_SWITCH_LAUNCH".to_string());
        }
        if self.is_shared() {
            text.push("SHARED".to_string());
        }
        if self.has_no_inits() {
            text.push("HAS_NO_INITS".to_string());
        }
        if self.has_been_inited() {
            text.push("INITED".to_string());
        }
        if self.has_custom_icon() {
            text.push("CUSTOM_ICON".to_string());
        }
        if self.is_stationery() {
            text.push("STATIONERY".to_string());
        }
        if self.name_locked() {
            text.push("NAME_LOCKED".to_string());
        }
        if self.has_bundle() {
            text.push("HAS_BUNDLE".to_string());
        }
        if self.is_invisible() {
            text.push("INVISIBLE".to_string());
        }
        if self.is_alias() {
            text.push("ALIAS".to_string());
        }
        write!(f, "{}", text.join("|"))
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

impl From<&[u8; 4]> for Point {
    fn from(bytes: &[u8; 4]) -> Self {
        let vertical = i16::from_be_bytes(*array_ref![bytes, 0, 2]);
        let horizontal = i16::from_be_bytes(*array_ref![bytes, 2, 2]);
        Self { vertical, horizontal }
    }
}

/// The ID of the window representing the folder containing this file
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Folder(u16);

impl From<&[u8; 2]> for Folder {
    fn from(bytes: &[u8; 2]) -> Self {
        Self(u16::from_be_bytes(*bytes))
    }
}

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

/// A bitfield data structure containing the "locked" and "protected" bits.
#[derive(Default, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct MacInfo(u32);

impl MacInfo {
    fn inner(&self) -> &BitSlice<u32, Lsb0> {
        self.0.view_bits()
    }
    pub fn is_locked(&self) -> bool {
        self.inner()[0]
    }
    pub fn is_protected(&self) -> bool {
        self.inner()[1]
    }
}

impl From<&[u8; 4]> for MacInfo {
    fn from(bytes: &[u8; 4]) -> Self {
        u32::from_be_bytes(*bytes).into()
    }
}

impl fmt::Display for MacInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = vec![];
        if self.is_locked() {
            text.push("LOCKED".to_string());
        }
        if self.is_protected() {
            text.push("PROTECTED".to_string());
        }
        write!(f, "{}", text.join("|"))
    }
}

impl fmt::Debug for MacInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MacInfo({})", self)
    }
}
