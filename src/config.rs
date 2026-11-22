use edn_format::Value;
use std::{collections::BTreeMap, fs, path::Path};

use crate::{DEFAULT_MAX_EPISODES, error::SteveError};

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub(crate) enum RssFeed {
    Url(String),
    Feed {
        url: String,
        max_episodes: Option<MaxEpisodes>,
        number: Option<bool>,
    },
}

impl RssFeed {
    pub(crate) fn number(&self) -> bool {
        matches!(
            self,
            RssFeed::Feed {
                number: Some(true),
                ..
            }
        )
    }

    pub(crate) fn get_url(&self) -> &str {
        match self {
            RssFeed::Url(url) => url,
            RssFeed::Feed {
                url,
                max_episodes: _,
                number: _,
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
                number: _,
            } => match max_episodes {
                Some(max) => max.max_episodes(default),
                None => Some(default),
            },
        }
    }
}

#[derive(Debug, Default)]
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
    parse_config(&raw)
}

fn parse_config(raw: &str) -> Result<SteveConfig, SteveError> {
    let value = edn_format::parse_str(raw).map_err(|source| SteveError::EdnParse { source })?;
    let map = expect_map(&value, "config root")?;
    let mut config = SteveConfig::default();

    for (key, value) in map {
        match key_name(key).as_deref() {
            Some("rss-urls") => config.rss_urls = parse_rss_urls(value)?,
            Some("episodes-dir") => {
                config.episodes_dir = expect_string(value, "episodes-dir")?.to_string();
            }
            Some("max-episodes") => {
                config.max_episodes = Some(expect_usize(value, "max-episodes")?);
            }
            _ => {}
        }
    }

    Ok(config)
}

fn parse_rss_urls(value: &Value) -> Result<Vec<RssFeed>, SteveError> {
    let values = match value {
        Value::Vector(values) => values,
        _ => return invalid("rss-urls must be a vector"),
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_rss_feed(value, index))
        .collect()
}

fn parse_rss_feed(value: &Value, index: usize) -> Result<RssFeed, SteveError> {
    match value {
        Value::String(url) => Ok(RssFeed::Url(url.clone())),
        Value::Map(map) => {
            let url = get_field(map, "url")
                .ok_or_else(|| SteveError::EdnConfig {
                    message: format!("rss-urls entry {index} is missing :url"),
                })
                .and_then(|value| {
                    expect_string(value, format!("rss-urls entry {index} url")).map(str::to_string)
                })?;

            let max_episodes = get_field(map, "max-episodes")
                .map(parse_max_episodes)
                .transpose()?;

            let number = match get_field(map, "number") {
                None | Some(Value::Nil) => None,
                Some(Value::Boolean(value)) => Some(*value),
                _ => {
                    return invalid(format!(
                        "rss-urls entry {index} number must be a boolean or nil"
                    ));
                }
            };

            Ok(RssFeed::Feed {
                url,
                max_episodes,
                number,
            })
        }
        _ => invalid(format!(
            "rss-urls entry {index} must be a string URL or map"
        )),
    }
}

fn parse_max_episodes(value: &Value) -> Result<MaxEpisodes, SteveError> {
    match value {
        Value::Integer(_) => expect_usize(value, "max-episodes").map(MaxEpisodes::Number),
        Value::String(text) => Ok(MaxEpisodes::Text(text.clone())),
        Value::Symbol(symbol) if symbol.name() == "all" && symbol.namespace().is_none() => {
            Ok(MaxEpisodes::Text("all".to_string()))
        }
        Value::Keyword(keyword) if keyword.name() == "all" && keyword.namespace().is_none() => {
            Ok(MaxEpisodes::Text("all".to_string()))
        }
        _ => invalid("max-episodes must be a positive integer, all, :all, or \"all\""),
    }
}

fn get_field<'a>(map: &'a BTreeMap<Value, Value>, field: &str) -> Option<&'a Value> {
    map.iter()
        .find(|(key, _)| key_name(key).as_deref() == Some(field))
        .map(|(_, value)| value)
}

fn key_name(value: &Value) -> Option<String> {
    match value {
        Value::Keyword(keyword) if keyword.namespace().is_none() => {
            Some(keyword.name().to_string())
        }
        Value::String(text) => Some(text.to_string()),
        _ => None,
    }
}

fn expect_map(
    value: &Value,
    context: impl AsRef<str>,
) -> Result<&BTreeMap<Value, Value>, SteveError> {
    match value {
        Value::Map(map) => Ok(map),
        _ => invalid(format!("{} must be a map", context.as_ref())),
    }
}

fn expect_string(value: &Value, context: impl AsRef<str>) -> Result<&str, SteveError> {
    match value {
        Value::String(text) => Ok(text),
        _ => invalid(format!("{} must be a string", context.as_ref())),
    }
}

fn expect_usize(value: &Value, context: impl AsRef<str>) -> Result<usize, SteveError> {
    match value {
        Value::Integer(value) => (*value).try_into().map_err(|_| SteveError::EdnConfig {
            message: format!("{} must be a non-negative integer", context.as_ref()),
        }),
        _ => invalid(format!("{} must be an integer", context.as_ref())),
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T, SteveError> {
    Err(SteveError::EdnConfig {
        message: message.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_edn_config() {
        let config = parse_config(
            r#"
            {:rss-urls ["https://example.com/feed.xml"
                        {:url "https://example.com/private.xml"
                         :max-episodes :all}]
             :episodes-dir "/home/me/episodes"
             :max-episodes 20}
            "#,
        )
        .unwrap();

        assert_eq!(config.episodes_dir, "/home/me/episodes");
        assert_eq!(config.max_episodes, Some(20));
        assert_eq!(config.rss_urls.len(), 2);
        assert_eq!(config.rss_urls[0].get_url(), "https://example.com/feed.xml");
        assert_eq!(
            config.rss_urls[1].get_url(),
            "https://example.com/private.xml"
        );
        assert_eq!(config.rss_urls[1].max_episodes(&Some(20)), None);
    }

    #[test]
    fn parses_optional_nullable_number() {
        let config = parse_config(
            r#"{:rss-urls ["https://example.com/feed"
                          {:url "feed"}
                          {:url "feed" :number nil}
                          {:url "feed" :number false}
                          {:url "feed" :number true}]}"#,
        )
        .unwrap();
        assert_eq!(
            config
                .rss_urls
                .iter()
                .map(RssFeed::number)
                .collect::<Vec<_>>(),
            vec![false, false, false, false, true]
        );
        let error = parse_config(r#"{:rss-urls [{:url "feed" :number 1}]}"#).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("number must be a boolean or nil")
        );
    }

    #[test]
    fn rejects_malformed_feed_entries() {
        let error = parse_config(r#"{:rss-urls [{:max-episodes 3}]}"#).unwrap_err();

        assert!(error.to_string().contains("missing :url"));
    }
}
