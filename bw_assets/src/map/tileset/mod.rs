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

pub use self::{cv5::*, vf4::*, vr4::*, vx4::*, wpe::*};
