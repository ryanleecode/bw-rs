//! Types and Parsers for the MPQ File Format
//!
//! MPQ are archives that are similar to zip files and directories; they contain
//! files inside of them. MPQ Archives are an efficient way to compress the data
//! and provide basic encryption.
//!
//! This module provides an implementation that opens an MPQ Archive and supports
//!

use amethyst::{
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use ceres_mpq::Archive;
use snafu::Snafu;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
    sync::Arc,
};

#[derive(Debug, Snafu)]
pub enum Error {
    /// Represents an error when reading a file on disk.
    #[snafu(display("failed to open file at path {}: {}", path, source))]
    OpenFile {
        source: std::io::Error,
        path: String,
    },

    /// Represents an error when trying to open an MPQ Archive.
    #[snafu(display("failed to open archive: {}", source))]
    OpenArchive { source: Box<Error> },

    /// Error that occurs when trying to read a file inside of an MPQ archive.
    #[snafu(display("failed to read file in archive: {}", source))]
    ReadFile { source: Box<Error> },

    /// Error that occurs from the ceres_mpq library.
    #[snafu(display("ceres mpq error: {}", source))]
    CeresMPQ { source: ceres_mpq::Error },

    #[snafu(display("io error: {}", source))]
    IO { source: std::io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

/// MPQ is an archive that we can read files from.
pub trait MPQ {
    fn read_file(&self, file_name: &str) -> Result<Vec<u8>>;
}

/// Newtype that wraps a ceres mpq archive and implements the read file trait.
struct ArchiveNewType<R: Read + Seek>(Archive<R>);

impl<R: Read + Seek> MPQ for ArchiveNewType<R> {
    fn read_file(&self, file_name: &str) -> Result<Vec<u8>> {
        use snafu::ResultExt;

        self.0
            .read_file(file_name)
            .context(CeresMPQ {})
            .map_err(Box::new)
            .context(ReadFile {})
    }
}

/// New Type around MPQ such we can load it in Amethyst and add it to MPQSource.
///
/// Just using MPQ is not enough because Amethyst's loader returns a reference that has
/// a limited lifetime.
pub struct ArcMPQ(Arc<dyn MPQ + Send + Sync>);

impl Clone for ArcMPQ {
    fn clone(&self) -> Self {
        ArcMPQ(self.0.clone())
    }
}

impl ArcMPQ {
    pub fn from_path(path: &Path) -> Result<ArcMPQ> {
        use snafu::ResultExt;

        let f = File::open(path).context(OpenFile {
            path: path.display().to_string(),
        })?;
        let br = BufReader::new(f);

        ArcMPQ::from_reader(br)
    }
}

impl ArcMPQ {
    pub fn new(mpq: impl MPQ + Send + Sync + 'static) -> ArcMPQ {
        ArcMPQ(Arc::new(mpq))
    }

    pub fn from_reader<R: Read + Seek + Send + Sync + 'static>(r: R) -> Result<ArcMPQ> {
        use snafu::ResultExt;

        ceres_mpq::Archive::open(r)
            .context(CeresMPQ {})
            .map_err(Box::new)
            .context(OpenArchive {})
            .map(ArchiveNewType)
            .map(ArcMPQ::new)
    }

    pub fn read_file(&self, file_name: &str) -> Result<Vec<u8>> {
        self.0.read_file(file_name)
    }
}

pub type MPQHandle = Handle<ArcMPQ>;

impl Asset for ArcMPQ {
    const NAME: &'static str = "bw_assets_mpq";
    type Data = Self;
    type HandleStorage = DenseVecStorage<MPQHandle>;
}

/// Amethyst asset source for a queue of MPQ files.
///
/// If multiple MPQ archives contain the same file, the archive at front of the
/// queue take precedence.
pub struct MPQSource {
    queue: VecDeque<ArcMPQ>,
}

impl MPQSource {
    pub fn new() -> MPQSource {
        MPQSource {
            queue: VecDeque::new(),
        }
    }

    pub fn push_back(&mut self, value: ArcMPQ) {
        self.queue.push_back(value)
    }

    pub fn push_front(&mut self, value: ArcMPQ) {
        self.queue.push_front(value)
    }
}

/// MPQ acts similar to a directory because it is an archive of files. We can
/// treat it as an Amethyst source.
impl amethyst::assets::Source for MPQSource {
    /// Modified is used for hot reloading but MPQ files will never change so we don't
    /// care and provide a default 0 value.
    fn modified(&self, _: &str) -> std::result::Result<u64, amethyst::Error> {
        Ok(0)
    }

    fn load(&self, file_name: &str) -> std::result::Result<Vec<u8>, amethyst::Error> {
        let mut errors = vec![];
        for mpq in self.queue.iter() {
            match mpq.read_file(file_name) {
                Ok(file) => return Ok(file),
                Err(err) => errors.push(err),
            }
        }

        Err(errors
            .into_iter()
            .fold(amethyst::error::Error::from_string(""), |err, source| {
                err.with_source(source)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::collections::HashMap;

    struct TestMPQ(HashMap<String, Vec<u8>>);

    impl MPQ for TestMPQ {
        fn read_file(
            &self,
            file_name: &str,
        ) -> std::result::Result<std::vec::Vec<u8>, super::Error> {
            use snafu::ResultExt;

            self.0
                .get(file_name)
                .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))
                .map(|v| v.clone())
                .context(IO {})
                .map_err(Box::new)
                .context(ReadFile {})
        }
    }

    #[test]
    fn mpq_at_the_front_should_take_precedence() {
        use amethyst::assets::Source;

        let mpq1 = ArcMPQ::new(TestMPQ(hashmap! {
            "file1".to_string() => vec![1,2,3],
        }));
        let mpq2 = ArcMPQ::new(TestMPQ(hashmap! {
            "file1".to_string() => vec![4,5,6],
        }));

        let mut mpq_source = MPQSource::new();
        mpq_source.push_front(mpq1);
        mpq_source.push_front(mpq2);

        assert_that(&mpq_source.load("file1"))
            .is_ok()
            .is_equal_to(vec![4, 5, 6])
    }
}
