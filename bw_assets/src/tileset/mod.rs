//! Types and Parsers for the Tileset File Formats
//!
//! Blizzard heavily optimized for space when they designed these formats.
//! Starcraft maps have an MXTM fields which reference at CV5 element. Each CV5
//! element represents a megatile (32x32 pixels) and references 16 8x8 minitiles
//! via VF4 and VX4 elements. VX4s indicate if the tile is flipped horizontally
//! and is also reference VR4. Each VR4 is a reference to 64 WPEs which represent
//! the color of the pixel. VF4 on the other hand show the gameplay flags such as
//! walkable, elevation, blocks view, etc...

mod cv5;
mod vf4;
mod vr4;
mod vx4;
mod wpe;

pub use self::cv5::{
    BuildFlag, CV5Data, CV5Format, CV5s, CV5sAsset, CV5sHandle, Doodad, MinitileReference,
    OverlayFlag, TileMetadata, CV5,
};
pub use self::vf4::{VF4Format, VF4s, VF4sAsset, VF4sHandle, VF4};
pub use self::vr4::{VR4Format, VR4s, VR4sAsset, VR4sHandle, VR4sIterator, VR4};
pub use self::vx4::{VX4s, VX4sAsset, VX4sAssetFormat, VX4sHandle, VX4};
pub use self::wpe::{WPEFormat, WPEs, WPEsAsset, WPEsHandle, WPE};
