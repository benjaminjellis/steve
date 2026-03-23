use std::{fs, path::PathBuf};

use clap::builder::styling::{AnsiColor, Color, Style};

use crate::error::SteveError;

pub(crate) fn list_dirs(dir: &PathBuf) -> Result<Vec<PathBuf>, SteveError> {
    Ok(fs::read_dir(dir)
        .map_err(|source| SteveError::IOError {
            source,
            context: format!("trying to read dir: {dir:?}"),
        })?
        .filter_map(|entry| match entry {
            Ok(e) => match e.file_type() {
                Ok(r) => {
                    if r.is_dir() {
                        Some(e.path())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
            Err(_) => None,
        })
        .collect::<Vec<_>>())
}

pub(crate) fn path_with_trailing_slash(path: PathBuf) -> String {
    let mut s = path.to_string_lossy().to_string();

    if !s.ends_with('/') {
        s.push('/');
    }

    s
}

pub(crate) fn available_workers() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        * 2
}

pub(crate) fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}
