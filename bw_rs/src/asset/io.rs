use anyhow::{Context, Result};
use bevy::{
    asset::{AssetIo, AssetIoError},
    utils::BoxedFuture,
};
use std::{
    collections::LinkedList,
    env,
    fs::File,
    io::{BufReader, Read, Seek},
    path::{Path, PathBuf},
};

pub(super) struct MPQAssetIO<R: Read + Seek> {
    archive: ceres_mpq::Archive<R>,
}

impl MPQAssetIO<BufReader<File>> {
    pub fn new<P: AsRef<Path>>(path: P, file_name: &str) -> Result<Self> {
        let root_path = Self::get_root_path().join(path.as_ref()).join(file_name);
        let root_path_display = format!("{}", root_path.display());
        let f = File::open(root_path)
            .with_context(|| format!("Failed to open `{}`.", root_path_display))?;
        let br = BufReader::new(f);
        let archive = ceres_mpq::Archive::open(br)?;

        Ok(MPQAssetIO { archive })
    }

    pub fn get_root_path() -> PathBuf {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
        } else {
            env::current_exe()
                .map(|path| {
                    path.parent()
                        .map(|exe_parent_path| exe_parent_path.to_owned())
                        .unwrap()
                })
                .unwrap()
        }
    }
}

impl<R: Read + Seek + Send + Sync + 'static> AssetIo for MPQAssetIO<R> {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            let os_path = path.to_string_lossy();
            self.archive.read_file(&os_path).map_err(|err| match err {
                ceres_mpq::Error::IoError { cause } => AssetIoError::Io(cause),
                ceres_mpq::Error::FileNotFound => AssetIoError::NotFound(path.to_path_buf()),
                _ => AssetIoError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
            })
        })
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        Err(AssetIoError::NotFound(path.to_path_buf()))
    }

    fn is_directory(&self, _: &Path) -> bool {
        false
    }

    fn watch_path_for_changes(&self, _: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }
}

pub(super) struct UnifiedMPQAssetIO {
    ll: LinkedList<Box<dyn AssetIo>>,
}

impl Default for UnifiedMPQAssetIO {
    fn default() -> Self {
        UnifiedMPQAssetIO {
            ll: LinkedList::default(),
        }
    }
}

impl UnifiedMPQAssetIO {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_source(&mut self, asset_io: impl AssetIo) {
        self.ll.push_front(Box::new(asset_io))
    }
}

impl AssetIo for UnifiedMPQAssetIO {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            for mpq_asset_io in &self.ll {
                match mpq_asset_io.load_path(path).await {
                    Ok(bytes) => return Ok(bytes),
                    Err(err) => match err {
                        AssetIoError::NotFound(_) => {
                            continue;
                        }
                        _ => return Err(err),
                    },
                }
            }

            Err(AssetIoError::NotFound(path.to_path_buf()))
        })
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        Err(AssetIoError::NotFound(path.to_path_buf()))
    }

    fn is_directory(&self, _: &Path) -> bool {
        false
    }

    fn watch_path_for_changes(&self, _: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }
}
