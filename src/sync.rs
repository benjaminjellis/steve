use crate::{MUSIC_CONTENT_DIR, ui};
use homedir::my_home;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{IPOD_CONTENT_DIR, IPOD_ROOT, PODCASTS_CONTENT_DIR, error::SteveError};

trait Plan {
    fn to_remove(&self) -> &Vec<PathBuf>;
    fn to_update(&self) -> &Vec<PathBuf>;
    fn to_skip(&self) -> &Vec<PathBuf>;
}

#[derive(Debug)]
struct PodcastSyncPlan {
    podcasts_to_remove: Vec<PathBuf>,
    podcasts_to_update: Vec<PathBuf>,
    podcasts_to_skip: Vec<PathBuf>,
}

impl PodcastSyncPlan {
    fn explain_plan(&self) {
        ui::yellow_std_out("Removing the following...".into());
        self.podcasts_to_remove
            .iter()
            .for_each(|to_remove| println!("{to_remove:?}"))
    }
}

impl Plan for PodcastSyncPlan {
    fn to_remove(&self) -> &Vec<PathBuf> {
        &self.podcasts_to_remove
    }

    fn to_update(&self) -> &Vec<PathBuf> {
        &self.podcasts_to_update
    }

    fn to_skip(&self) -> &Vec<PathBuf> {
        &self.podcasts_to_skip
    }
}

#[derive(Debug)]
struct MusicSyncPlan {
    music_to_remove: Vec<PathBuf>,
    music_to_update: Vec<PathBuf>,
    music_to_skip: Vec<PathBuf>,
}

impl MusicSyncPlan {
    fn explain_plan(&self) {
        ui::yellow_std_out("Removing the following...".into());
        self.music_to_remove
            .iter()
            .for_each(|to_remove| println!("{to_remove:?}"))
    }
}

impl Plan for MusicSyncPlan {
    fn to_remove(&self) -> &Vec<PathBuf> {
        &self.music_to_remove
    }

    fn to_update(&self) -> &Vec<PathBuf> {
        &self.music_to_update
    }

    fn to_skip(&self) -> &Vec<PathBuf> {
        &self.music_to_skip
    }
}

fn sync_music(
    dry_run: bool,
    local_content_dir: PathBuf,
    ipod_content_dir: PathBuf,
) -> Result<(), SteveError> {
    let music_dir = local_content_dir.join(MUSIC_CONTENT_DIR);

    let ipod_music_dir = ipod_content_dir.join(MUSIC_CONTENT_DIR);

    let artist_dirs = crate::utils::list_dirs(&music_dir)?;
    let artists_on_ipod = crate::utils::list_dirs(&ipod_music_dir)?;

    let artists = artist_dirs
        .iter()
        .filter_map(|a| a.file_name())
        .collect::<Vec<_>>();

    let mut plan = MusicSyncPlan {
        music_to_remove: vec![],
        music_to_update: vec![],
        music_to_skip: vec![],
    };

    for artist in artists_on_ipod {
        if let Some(artist_on_ipod_name) = artist.file_name()
            && !artists.contains(&artist_on_ipod_name)
        {
            plan.music_to_remove.push(artist);
        }
    }
    plan.music_to_update = artist_dirs;

    plan.explain_plan();

    if !dry_run {
        execute_plan(plan, ipod_music_dir)?;
    }

    Ok(())
}

fn sync_podcasts(
    dry_run: bool,
    local_content_dir: &Path,
    ipod_content_dir: &Path,
) -> Result<(), SteveError> {
    let podcasts_dir = local_content_dir.join(PODCASTS_CONTENT_DIR);

    let ipod_podcasts_dir = ipod_content_dir.join(PODCASTS_CONTENT_DIR);
    let podcasts_file_paths = crate::utils::list_dirs(&podcasts_dir)?;
    let podcasts_on_ipod_paths = crate::utils::list_dirs(&ipod_podcasts_dir)?;

    let podcast_names = podcasts_file_paths
        .iter()
        .filter_map(|a| a.file_name())
        .collect::<Vec<_>>();

    let mut plan = PodcastSyncPlan {
        podcasts_to_remove: vec![],
        podcasts_to_update: vec![],
        // TODO: add skip to config
        podcasts_to_skip: vec![PathBuf::from(
            "/Users/ben/iPod Content/Podcasts/Podcast About List Premium",
        )],
    };

    for file_path in podcasts_on_ipod_paths {
        if let Some(podcast_on_ipod_name) = file_path.file_name()
            && !podcast_names.contains(&podcast_on_ipod_name)
        {
            plan.podcasts_to_remove.push(file_path);
        }
    }

    plan.podcasts_to_update = podcasts_file_paths;

    plan.explain_plan();

    if !dry_run {
        execute_plan(plan, ipod_podcasts_dir)?;
    }
    Ok(())
}

pub(crate) fn run_sync(dry_run: bool) -> Result<(), SteveError> {
    let Some(home_dir) = my_home().map_err(|source| SteveError::NoHomeDir { source })? else {
        ui::red_std_err("No home directory found".into());
        return Ok(());
    };

    let local_content_dir = home_dir.join(IPOD_CONTENT_DIR);
    let ipod_content_dir = PathBuf::from(IPOD_ROOT);

    if !ipod_content_dir.is_dir() {
        return Err(SteveError::NoConnectedIpod {
            path: ipod_content_dir,
        });
    }
    sync_podcasts(dry_run, &local_content_dir, &ipod_content_dir)?;
    sync_music(dry_run, local_content_dir, ipod_content_dir)?;

    Ok(())
}

fn execute_plan<P: Plan>(plan: P, content_dir: PathBuf) -> Result<(), SteveError> {
    for to_remove in plan.to_remove() {
        println!("Removing {to_remove:?}");
        std::fs::remove_dir_all(to_remove).unwrap();
    }

    for to_update in plan.to_update() {
        if let Some(podcast_name) = to_update.file_name()
            && !plan.to_skip().contains(to_update)
        {
            let ipod_podcast_dir = content_dir.join(podcast_name);
            let source = crate::utils::path_with_trailing_slash(to_update);
            let dest = crate::utils::path_with_trailing_slash(&ipod_podcast_dir);
            // Prefer speed: compare by file size only (ignore timestamps/metadata).
            let status = Command::new("rsync")
                .args([
                    "-rtv",
                    "--size-only",
                    "--delete",
                    "--exclude=._*",
                    "--exclude=.DS_Store",
                    "--progress",
                    source.as_str(),
                    dest.as_str(),
                ])
                .status()
                .unwrap();
            if !status.success() {
                eprintln!("rsync failed with: {}", status);
            }
        }
    }
    Ok(())
}
