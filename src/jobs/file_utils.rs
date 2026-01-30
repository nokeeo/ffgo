use crate::path_strings::PathStrings;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn get_jobs(dir: &Path) -> HashMap<String, Vec<PathBuf>> {
    let mut jobs = HashMap::<String, Vec<PathBuf>>::new();
    let paths = get_paths_in_dir(dir).into_iter().filter(|p| !is_config(p));
    for path in paths {
        let Some(job_id) = get_job_id(path.as_path()) else {
            println!("Failed to get path for {:?}", path);
            continue;
        };
        let entry = jobs.entry(job_id.to_owned()).or_insert(Vec::<PathBuf>::new());
        entry.push(path);
    }
    jobs
}

fn get_job_id(file: &Path) -> Option<&str> {
    file.file_stem_str()?.split('_').next()
}

fn get_paths_in_dir(path: &Path) -> Vec<PathBuf> {
    return std::fs::read_dir(path).unwrap().filter_map(|entry| {
        match entry {
            Ok(entry) => Some(entry.path()),
            _ => None
        }
    }).collect::<Vec<PathBuf>>();
}

fn is_config(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) &&
        extension == "toml" {
        return true;
    }

    let Some(file_stem) = path.file_stem_str() else {
        return false;
    };
    file_stem.starts_with(".")
}

