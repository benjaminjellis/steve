use crate::ui;
use std::{path::PathBuf, process::Command};

use crate::{IPOD_CONTENT_DIR, IPOD_ROOT, PODCASTS_CONTENT_DIR, error::SteveError};

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

pub(crate) fn run_sync(dry_run: bool) -> Result<(), SteveError> {
    let local_content_dir = PathBuf::from(IPOD_CONTENT_DIR);
    let ipod_content_dir = PathBuf::from(IPOD_ROOT);

    if !ipod_content_dir.is_dir() {
        return Err(SteveError::NoConnectedIpod {
            path: ipod_content_dir,
        });
    }

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

fn execute_plan(plan: PodcastSyncPlan, ipod_podcasts_dir: PathBuf) -> Result<(), SteveError> {
    for to_remove in plan.podcasts_to_remove {
        println!("Removing {to_remove:?}");
        std::fs::remove_dir_all(&to_remove).unwrap();
    }

    for to_update in plan.podcasts_to_update {
        if let Some(podcast_name) = to_update.file_name()
            && !plan.podcasts_to_skip.contains(&to_update)
        {
            let ipod_podcast_dir = ipod_podcasts_dir.join(podcast_name);
            let source = crate::utils::path_with_trailing_slash(to_update);
            let dest = crate::utils::path_with_trailing_slash(ipod_podcast_dir);
            // use rysnc to copy
            let status = Command::new("rsync")
                .args([
                    "-rtv",
                    "--delete",
                    "--modify-window=1",
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
