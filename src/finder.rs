use std::{
    fmt,
    num::NonZeroI8,
};

use derive_more::{From, Into};

use four_cc::FourCC as ForeignFourCC;
use deku::{prelude::*, bitvec::{BitSlice, Msb0}};

#[derive(DekuRead, DekuWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderInfo {
    pub file_type: FileType,
    pub creator: Creator,
    pub flags: FinderFlags,
    pub location: Point,
    pub folder: Folder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FourCC(ForeignFourCC);

impl<'a, C> DekuRead<'a, C> for FourCC where C: Copy, u8: DekuRead<'a, C> {
    fn read(
        input: &'a BitSlice<u8, Msb0>,
        ctx: C,
    ) -> Result<(&'a BitSlice<u8, Msb0>, Self), DekuError>
        where Self: Sized {
        let (rest, bytes): (_, [u8; 4]) = DekuRead::read(input, ctx)?;
        let fourcc = ForeignFourCC(bytes);
        Ok((rest, Self(fourcc)))
    }
}

impl<C> DekuWrite<C> for FourCC where C: Copy, u8: deku::DekuWrite<C> {
    fn write(
        &self,
        output: &mut deku::bitvec::BitVec<u8, Msb0>,
        ctx: C,
    ) -> Result<(), DekuError> {
        DekuWrite::write(&self.0.0, output, ctx)
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Mac File Type code
#[derive(Debug, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq)]
pub struct FileType(FourCC);

/// Mac Creator code
#[derive(Debug, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq)]
pub struct Creator(FourCC);

/// Various flags that are either manipulated by the Finder or influence the way
/// the Finder will present the file.
#[derive(DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq)]
#[deku(endian="big")]
pub struct FinderFlags {
    #[deku(bits = "1")]
    pub is_alias: bool,
    #[deku(bits = "1")]
    pub is_invisible: bool,
    #[deku(bits = "1")]
    pub has_bundle: bool,
    #[deku(bits = "1")]
    pub name_locked: bool,
    #[deku(bits = "1")]
    pub is_stationery: bool,
    #[deku(bits = "1", pad_bits_after = "1")]
    pub has_custom_icon: bool,

    #[deku(bits = "1")]
    pub has_been_inited: bool,
    #[deku(bits = "1")]
    pub has_no_inits: bool,
    #[deku(bits = "1")]
    pub is_shared: bool,
    #[deprecated]
    #[deku(bits = "1", pad_bits_after = "1")]
    pub requires_switch_launch: bool,

    #[deku(bits = "3")]
    pub color: u8,
    #[deku(bits = "1")]
    #[deprecated]
    pub is_on_desktop: bool,
}

impl fmt::Display for FinderFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = vec![];
        #[allow(deprecated)]
        if self.is_on_desktop {
            text.push("ON_DESKTOP".to_string());
        }
        text.push(format!("COLOR={}", self.color));
        #[allow(deprecated)]
        if self.requires_switch_launch {
            text.push("REQUIRES_SWITCH_LAUNCH".to_string());
        }
        if self.is_shared {
            text.push("SHARED".to_string());
        }
        if self.has_no_inits {
            text.push("HAS_NO_INITS".to_string());
        }
        if self.has_been_inited {
            text.push("INITED".to_string());
        }
        if self.has_custom_icon {
            text.push("CUSTOM_ICON".to_string());
        }
        if self.is_stationery {
            text.push("STATIONERY".to_string());
        }
        if self.name_locked {
            text.push("NAME_LOCKED".to_string());
        }
        if self.has_bundle {
            text.push("HAS_BUNDLE".to_string());
        }
        if self.is_invisible {
            text.push("INVISIBLE".to_string());
        }
        if self.is_alias {
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

/// A 2-dimensional point in QuickDraw's coordinate system
#[derive(Debug, DekuRead, DekuWrite, Default, Clone, Copy, PartialEq, Eq)]
#[deku(endian = "big")]
pub struct Point {
    #[deku(bits = "16")]
    vertical: i16,
    #[deku(bits = "16")]
    horizontal: i16,
}

/// The ID of the window representing the folder containing this file
#[derive(Debug, DekuRead, DekuWrite, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[deku(endian = "big")]
pub struct Folder(#[deku(bits = "16")] u16);

/// A bunch of extra information which is not very useful to the typical
/// developer.
#[derive(Debug, DekuRead, DekuWrite, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedFinderInfo {
    icon_id: i16,
    filename_script: FilenameScript,
    comment_id: i16,
    put_away_from: i32,
}

/// The script used to display the filename. If unspecified, then the finder
/// should use whatever the user currently is using.
#[derive(Debug, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq)]
#[deku(type = "i8")]
pub enum FilenameScript {
    #[deku(id = "0")]
    Unspecified,
    #[deku(id_pat = "_")]
    Script(NonZeroI8),
}

impl Default for FilenameScript {
    fn default() -> Self {
        Self::Unspecified
    }
}

/// A bitfield data structure containing the "locked" and "protected" bits.
#[derive(Default, DekuRead, DekuWrite, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct MacInfo {
    #[deku(bits = "1", pad_bits_before = "30")]
    pub is_protected: bool,
    #[deku(bits = "1")]
    pub is_locked: bool,
}

impl fmt::Display for MacInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = vec![];
        if self.is_locked {
            text.push("LOCKED".to_string());
        }
        if self.is_protected {
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
