use std::path::PathBuf;

use homedir::GetHomeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SteveError {
    #[error("IOError: {context}")]
    IOError {
        source: std::io::Error,
        context: String,
    },
    #[error("Unable to find a connected iPod at path: {path:?}")]
    NoConnectedIpod { path: PathBuf },
    #[error("Unable to deserialize config as TOML: {source:?}")]
    TomlDeserialzie { source: toml::de::Error },
    #[error("Unable to find homedir: {source:?}")]
    NoHomeDir { source: GetHomeError },
}
