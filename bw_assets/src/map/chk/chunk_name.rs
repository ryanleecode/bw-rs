use std::{fmt, str};

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkName {
    Type,
    Version,
    Tileset,
    Controllers,
    Dimensions,
    Side,
    MegaTiles,
    StringData,
    Unit,
    Unknown,
}

impl fmt::Display for ChunkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        unsafe { write!(f, "{}", str::from_utf8_unchecked(self.as_bytes())) }
    }
}

impl ChunkName {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            ChunkName::Type => "TYPE".as_bytes(),
            ChunkName::Version => "VER ".as_bytes(),
            ChunkName::Tileset => "ERA ".as_bytes(),
            ChunkName::Controllers => "OWNR".as_bytes(),
            ChunkName::Dimensions => "DIM ".as_bytes(),
            ChunkName::Side => "SIDE".as_bytes(),
            ChunkName::MegaTiles => "MTXM".as_bytes(),
            ChunkName::StringData => "STR ".as_bytes(),
            ChunkName::Unit => "UNIT".as_bytes(),
            ChunkName::Unknown => "????".as_bytes(),
        }
    }
}
