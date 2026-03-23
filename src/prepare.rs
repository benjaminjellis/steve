use std::{
    collections::HashSet,
    env,
    fs::File,
    io,
    path::{Path, PathBuf},
};

use crate::ui;
use icu_normalizer::DecomposingNormalizerBorrowed;
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelRefIterator, ParallelIterator},
};
use reqwest::blocking::Client;
use rss::Channel;
use std::fs;

use crate::{AUDIO_EXTENSIONS, config::RssFeed, error::SteveError, utils::available_workers};

fn env_nonblank(name: &str) -> Option<String> {
    let value = env::var(name).ok()?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn config_path() -> PathBuf {
    if let Some(xdg) = env_nonblank("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("steve").join("config.toml")
    } else if let Some(home) = env_nonblank("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("steve")
            .join("config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

fn parse_rss(
    client: &Client,
    feed: &RssFeed,
    default_max_episodes: &Option<usize>,
) -> Result<(String, Vec<Episode>, Option<usize>), SteveError> {
    let response = client
        .get(feed.get_url())
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();

    let bytes = response.bytes().unwrap();

    let channel = Channel::read_from(&bytes[..]).unwrap();

    let title = {
        let t = channel.title().trim();
        if t.is_empty() {
            "unknown-feed".to_string()
        } else {
            t.to_string()
        }
    };

    let mut episodes: Vec<Episode> = channel
        .items()
        .iter()
        .filter_map(|item| {
            let enclosure = item.enclosure()?;
            let url = enclosure.url().trim();
            if url.is_empty() {
                return None;
            }
            Some(Episode {
                title: item.title().unwrap_or("untitled").to_string(),
                url: url.to_string(),
            })
        })
        .collect();
    let max_episodes = feed.max_episodes(default_max_episodes);

    if let Some(limit) = max_episodes {
        episodes.truncate(limit);
    }

    Ok((title, episodes, max_episodes))
}

fn has_supported_audio_extension(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    AUDIO_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn episode_filenames(episodes: &[Episode]) -> HashSet<String> {
    episodes
        .iter()
        .map(|ep| filename_from(&ep.title, &ep.url))
        .collect()
}

fn prune_old_episodes(
    podcast_dir: &Path,
    episodes: &[Episode],
    max_episodes: Option<usize>,
    dry_run: bool,
) -> Result<(), SteveError> {
    if max_episodes.is_none() {
        return Ok(());
    }

    let filenames_to_keep = episode_filenames(episodes);
    let entries = match fs::read_dir(podcast_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(SteveError::IOError {
                source,
                context: "".into(),
            });
        }
    };

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };

        if !has_supported_audio_extension(name) || filenames_to_keep.contains(name) {
            continue;
        }

        if dry_run {
            ui::green_std_out(format!("Would delete old episode: {name}"));
            continue;
        }

        match fs::remove_file(&path) {
            Ok(()) => println!("Deleted old episode: {name}"),
            Err(_) => println!("Failed deleting old episode: {}", path.display()),
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct DownloadJob {
    url: String,
    filepath: PathBuf,
}

fn extension_from_url(url: &str) -> &'static str {
    let lower = url.to_ascii_lowercase();
    for ext in AUDIO_EXTENSIONS {
        let needle = format!(".{ext}");
        if lower.contains(&needle) {
            return ext;
        }
    }
    "mp3"
}

fn normalize_nfd(input: &str) -> String {
    DecomposingNormalizerBorrowed::new_nfd()
        .normalize(input)
        .into_owned()
}

fn filename_from(title: &str, url: &str) -> String {
    format!("{}.{}", sanitize_filename(title), extension_from_url(url))
}

fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_space = false;

    for ch in input.chars() {
        let mapped = if ch.is_control()
            || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        {
            ' '
        } else {
            ch
        };

        if mapped.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(mapped);
            prev_space = false;
        }
    }

    let cleaned = out.trim();
    if cleaned.is_empty() {
        "untitled".to_string()
    } else {
        normalize_nfd(cleaned)
    }
}

#[derive(Debug, Clone)]
struct Episode {
    title: String,
    url: String,
}

fn download_episodes(
    client: &Client,
    feed_title: &str,
    episodes_dir: &Path,
    episodes: &[Episode],
    download_workers: usize,
    dry_run: bool,
    max_episodes: Option<usize>,
) -> Result<(), SteveError> {
    if !dry_run {
        fs::create_dir_all(episodes_dir).unwrap();
    }

    let podcast_dir = episodes_dir.join(sanitize_filename(feed_title));
    if !dry_run {
        fs::create_dir_all(&podcast_dir).unwrap();
    }

    let jobs: Vec<DownloadJob> = episodes
        .iter()
        .filter_map(|ep| {
            let filename = filename_from(&ep.title, &ep.url);
            let filepath = podcast_dir.join(&filename);
            if filepath.exists() {
                println!("  Skipping (exists): {filename}");
                None
            } else {
                Some(DownloadJob {
                    url: ep.url.clone(),
                    filepath,
                })
            }
        })
        .collect();

    if !jobs.is_empty() {
        if dry_run {
            for job in &jobs {
                println!("Would download: {} -> {}", job.url, job.filepath.display());
            }
        } else {
            let pool = ThreadPoolBuilder::new()
                .num_threads(download_workers)
                .build()
                .unwrap();
            let client = client.clone();
            pool.install(|| {
                jobs.par_iter()
                    .try_for_each(|job| download_file(&client, &job.url, &job.filepath))
            })
            .unwrap()
        }
    }

    prune_old_episodes(&podcast_dir, episodes, max_episodes, dry_run)?;
    Ok(())
}

fn download_file(client: &Client, url: &str, filepath: &Path) -> Result<(), SteveError> {
    println!("Downloading: {url}");
    let mut response = client.get(url).send().unwrap().error_for_status().unwrap();

    let mut out = File::create(filepath).unwrap();
    io::copy(&mut response, &mut out).unwrap();
    println!("Saved to: {}", filepath.display());
    Ok(())
}

pub(crate) fn run_prepare(dry_run: bool) -> Result<(), SteveError> {
    let config_path = config_path();
    let config = crate::config::read_config(&config_path)?;
    let worker_count = available_workers();

    if dry_run {
        println!("Dry run mode enabled: no files will be downloaded or deleted.");
    }
    println!("Download workers: {worker_count}");

    let client = Client::builder()
        .user_agent("steve-rust/0.1")
        .build()
        .unwrap();

    let episodes_dir = PathBuf::from(config.episodes_dir);

    for feed in &config.rss_urls {
        let (feed_title, episodes, max_episodes) = parse_rss(&client, feed, &config.max_episodes)?;

        ui::green_std_out(format!("fetching episodes of {feed_title}"));
        download_episodes(
            &client,
            &feed_title,
            &episodes_dir,
            &episodes,
            worker_count,
            dry_run,
            max_episodes,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::prepare::{filename_from, sanitize_filename};

    #[test]
    fn sanitize_filename_test() {
        assert_eq!("hello world.mp3", sanitize_filename("hello world.mp3"));
        assert_eq!("A B C", sanitize_filename("A/B/C"));
        assert_eq!(
            "Episode 531 Epstein The Movies",
            sanitize_filename("Episode 531: Epstein: The Movies")
        );
        assert_eq!("Se\u{301}amus", sanitize_filename("Séamus"));
        assert_eq!("untitled", sanitize_filename(" \n\t "));
    }

    #[test]
    fn filename_from_test() {
        assert_eq!(
            "Episode 1.mp3",
            filename_from("Episode 1", "https://cdn.example.com/ep1.mp3")
        );
        assert_eq!(
            "Episode 2.aac",
            filename_from("Episode 2", "https://cdn.example.com/ep2.aac")
        );
    }
}
