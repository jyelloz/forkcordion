use std::num::NonZeroI8;

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

/// Mac File Type code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileType(FourCC);

/// Mac Creator code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Creator(FourCC);

/// Various flags that are either manipulated by the Finder or influence the way
/// the Finder will present the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderFlags {
    inner: BitArr!(for 16, in Msb0, u16),
}

impl FinderFlags {
    #[deprecated]
    pub fn is_on_desktop(&self) -> bool {
        self.inner[0]
    }
    pub fn color(&self) -> u8 {
        self.inner[1..4].load_be()
    }
    #[deprecated]
    pub fn color_reserved(&self) -> bool {
        self.inner[4]
    }
    #[deprecated]
    pub fn requires_switch_launch(&self) -> bool {
        self.inner[5]
    }
    pub fn is_shared(&self) -> bool {
        self.inner[6]
    }
    pub fn has_no_inits(&self) -> bool {
        self.inner[7]
    }
    pub fn has_been_inited(&self) -> bool {
        self.inner[8]
    }
    pub fn has_custom_icon(&self) -> bool {
        self.inner[10]
    }
    pub fn is_stationery(&self) -> bool {
        self.inner[11]
    }
    pub fn name_locked(&self) -> bool {
        self.inner[12]
    }
    pub fn has_bundle(&self) -> bool {
        self.inner[13]
    }
    pub fn is_invisible(&self) -> bool {
        self.inner[14]
    }
    pub fn is_alias(&self) -> bool {
        self.inner[15]
    }
}

/// A 2-dimensional point in QuickDraw's coordinate system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    vertical: i16,
    horizontal: i16,
}

/// The ID of the window representing the folder containing this file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Folder(u16);

/// A bunch of extra information which is not very useful to the typical
/// developer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
