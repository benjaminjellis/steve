use std::path::PathBuf;

use homedir::GetHomeError;
use rayon::ThreadPoolBuildError;
use reqwest::StatusCode;
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
    #[error("Encountered a reqwest client error: {source:?}")]
    ReqwestClientError { source: reqwest::Error, url: String },
    #[error(
        "Encountered a http error code for request: {status_code:?} for url {url} when {context}"
    )]
    HttpErrorStatusCode {
        status_code: Option<StatusCode>,
        url: String,
        context: &'static str,
    },
    #[error("Encountered an error when trying to get http request body as bytes: {source:?}")]
    HttpResponseBytes { source: reqwest::Error },
    #[error("Failed to parse fetched rss feed as a channel: {source:?}")]
    RssChanelRead { source: rss::Error },
    #[error("Failed to create directories: {source:?}")]
    CreateDirs { source: std::io::Error },
    #[error("Failed to build HTTP client: {source:?}")]
    HttpClientBuild { source: reqwest::Error },
    #[error("Failed to build download thread pool: {source}")]
    ThreadPoolBuild { source: ThreadPoolBuildError },
}
