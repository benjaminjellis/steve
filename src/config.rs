use serde::Deserialize;
use std::path::Path;

use crate::{DEFAULT_MAX_EPISODES, error::SteveError};
use std::fs;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum MaxEpisodes {
    Number(usize),
    Text(String),
}

impl MaxEpisodes {
    fn max_episodes(&self, default: usize) -> Option<usize> {
        match self {
            MaxEpisodes::Number(max) => Some(*max),
            MaxEpisodes::Text(text) => {
                if text == "all" {
                    None
                } else {
                    // TODO: log here that the config is bad
                    Some(default)
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RssFeed {
    Url(String),
    Feed {
        url: String,
        #[serde(rename = "max-episodes")]
        max_episodes: Option<MaxEpisodes>,
    },
}

impl RssFeed {
    pub(crate) fn get_url(&self) -> &str {
        match self {
            RssFeed::Url(url) => url,
            RssFeed::Feed {
                url,
                max_episodes: _,
            } => url,
        }
    }

    /// Get the max number of episodes to download, if the None then all episodes should be fetched
    pub(crate) fn max_episodes(&self, user_specified_default: &Option<usize>) -> Option<usize> {
        let default = user_specified_default.unwrap_or(DEFAULT_MAX_EPISODES);
        match self {
            RssFeed::Url(_) => Some(default),
            RssFeed::Feed {
                url: _,
                max_episodes,
            } => match max_episodes {
                Some(max) => max.max_episodes(default),
                None => Some(default),
            },
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SteveConfig {
    pub(crate) rss_urls: Vec<RssFeed>,
    pub(crate) episodes_dir: String,
    pub(crate) max_episodes: Option<usize>,
}

pub(crate) fn read_config(path: &Path) -> Result<SteveConfig, SteveError> {
    if !path.exists() {
        return Ok(SteveConfig::default());
    }
    let raw = fs::read_to_string(path).unwrap();
    toml::from_str(&raw).map_err(|source| SteveError::TomlDeserialzie { source })
}
