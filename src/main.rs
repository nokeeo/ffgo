use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command; 

fn is_config(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) &&
        extension == "toml" {
        return true;
    }

    let stem = path.file_stem().expect("error").to_str().expect("error");
    stem.starts_with(".")
}

fn get_paths_in_dir(path: &Path) -> Vec<PathBuf> {
    return std::fs::read_dir(path).unwrap().filter_map(|entry| {
        match entry {
            Ok(entry) => Some(entry.path()),
            _ => None
        }
    }).collect::<Vec<PathBuf>>();
}

#[derive(Debug, Deserialize)]
struct OutputConfig {
    extension: String,
    directory: String
}

#[derive(Debug, Deserialize)]
struct Config {
    args: Vec<String>,
    output: OutputConfig,
}

impl Config {
    fn new_command(&self, files: &Vec<PathBuf>) -> Command {
        let mut command = Command::new("ffmpeg");
        self.add_input_files(&mut command, files);
        self.add_args(&mut command);
        self.add_output_arg(&mut command, files.first().unwrap());

        println!("{:?}", command.get_args());
        command
    }

    fn add_input_files(&self, command: &mut Command, files: &Vec<PathBuf>) {
        for path in files {
            command.args(["-i", path.to_str().unwrap()]);
        }
    }

    fn add_args(&self, command: &mut Command) {
        for arg in self.args.iter() {
            for part in arg.split(' ').collect::<Vec<&str>>() {
                command.arg(part);
            }
        }
    }

    fn add_output_arg(&self, command: &mut Command, path: &PathBuf) { 
        let filename = path.file_stem().expect("Faild to get file stem").to_str().expect("Failed to convert OSString to string");
        command.arg(format!("{}/{}.{}", self.output.directory, filename, self.output.extension));
    }
}

fn get_job_id(file: &Path) -> String {
    return file.file_stem().expect("error").to_str().expect("error").split("_").nth(0).unwrap().to_owned();
}

fn get_jobs(dir: &Path) -> HashMap<String, Vec<PathBuf>> {
    let mut jobs = HashMap::<String, Vec<PathBuf>>::new();
    let paths = get_paths_in_dir(dir).into_iter().filter(|p| !is_config(p));
    for path in paths {
        let job_id = get_job_id(path.as_path());
        let entry = jobs.entry(job_id).or_insert(Vec::<PathBuf>::new());
        entry.push(path);
    }
    jobs
}

fn main() -> Result<(), Box<dyn Error>> { 
    let args: Vec<String> = env::args().collect();
    let mut dir_path = PathBuf::new();
    dir_path.push(&args[1]);
    let config = fs::read_to_string(format!("{}/job.toml", &args[1])).expect("Can't open job.toml");
    let config: Config = toml::from_str(&config)?;

    for (_, mut files) in get_jobs(dir_path.as_path()) {
        files.sort();
        let output = config.new_command(&files)
            .output()
            .expect("Failed to execute ffmpeg");
        println!("{:?}", output);
    }
    Ok(())
}
