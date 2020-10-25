//! Types and Parsers for Starcraft Broodwar Asset Formats
//!
//! This crate provides the types and parsers necessary to process Brood War
//! assets. First class support is provided to load these assets using the
//! Amethyst game engine.

#[macro_use]
extern crate derive_builder;
#[macro_use]
#[cfg(test)]
extern crate maplit;

pub mod map;
pub mod mpq;
