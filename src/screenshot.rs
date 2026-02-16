use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Local;

pub fn save_screenshot(dir: &Path, frame: &[String]) -> io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let name = format!("watch-{}.txt", Local::now().format("%Y%m%d-%H%M%S"));
    let path = dir.join(name);
    let mut file = std::fs::File::create(&path)?;
    for line in frame {
        writeln!(file, "{line}")?;
    }
    Ok(path)
}
