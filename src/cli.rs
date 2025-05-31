use clap::Parser;
use std::path::PathBuf;

/// Copy DVD CLI Arguments
#[derive(Parser, Debug)]
#[clap(
    name = "Copy DVD",
    about = "A multi-threaded DVD copying application with CLI support and optional GUI - copy DVDs to MP4 and other formats",
    version,
    after_help = "Note: GUI mode is available as an optional feature but may have rendering issues on some macOS systems."
)]
pub struct Args {
    /// Path to DVD drive or directory
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// Output directory or file path
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Rip only the main feature (longest title)
    #[clap(long, conflicts_with = "titles")]
    pub main_feature: bool,

    /// Specific titles to rip (comma-separated, e.g., "1,2,3")
    #[clap(short, long, value_parser = parse_titles)]
    pub titles: Option<Vec<usize>>,

    /// Split chapters into separate files
    #[clap(short, long)]
    pub chapter_split: Option<bool>,

    /// Number of simultaneous ripping threads
    #[clap(short = 'n', long)]
    pub threads: Option<usize>,

    /// Scan DVD and display title information
    #[clap(long)]
    pub scan: bool,

    /// Eject DVD after ripping
    #[clap(long)]
    pub eject: bool,

    /// Encoding algorithm to use (x264 or x265)
    #[clap(long, default_value = "x264")]
    pub encoder: String,

    /// Force CLI mode (no GUI)
    #[clap(short = 'C', long = "cli", action = clap::ArgAction::SetTrue)]
    pub cli_mode: bool, // Changed to bool, defaults to false

    /// Upload to server after ripping (requires server configuration)
    #[clap(long)]
    pub upload: bool,
}

/// Parse comma-separated title numbers
fn parse_titles(s: &str) -> Result<Vec<usize>, String> {
    let mut titles = Vec::new();

    for part in s.split(',') {
        if part.contains('-') {
            // Handle ranges like 1-3
            let range: Vec<&str> = part.split('-').collect();
            if range.len() != 2 {
                return Err(format!("Invalid title range: {}", part));
            }

            let start: usize = range[0]
                .parse()
                .map_err(|_| format!("Invalid title number: {}", range[0]))?;
            let end: usize = range[1]
                .parse()
                .map_err(|_| format!("Invalid title number: {}", range[1]))?;

            if start > end {
                return Err(format!("Invalid title range: {}-{}", start, end));
            }

            titles.extend(start..=end);
        } else {
            // Handle individual numbers
            let title: usize = part
                .parse()
                .map_err(|_| format!("Invalid title number: {}", part))?;
            titles.push(title);
        }
    }

    Ok(titles)
}

/// Parse command-line arguments
pub fn parse_args() -> Args {
    Args::parse()
}
