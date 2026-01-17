use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command; 

fn is_video_file(path: &PathBuf) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        return match extension {
            "mov" | "m4v" | "mkv" | "mp4" => true,
            _ => false
        };
    }
    false
}

fn is_subtitle_file(path: &PathBuf) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        return match extension {
            "srt" | "vtt" | "ssa" | "scc" | "stl" => true,
            _ => false
        };
    }
    false
}

fn get_paths_in_dir(path: &PathBuf) -> Vec<PathBuf> {
    return std::fs::read_dir(path).unwrap().filter_map(|entry| {
        match entry {
            Ok(entry) => Some(entry.path()),
            _ => None
        }
    }).collect::<Vec<PathBuf>>();
}

fn get_video_subtitle_files(dir: &PathBuf, video_path: &PathBuf) -> Vec<PathBuf> {
    let video_stem = video_path.file_stem().expect("error").to_str().expect("error");
    return get_paths_in_dir(dir).into_iter().filter(|p| {
        if !is_subtitle_file(p) { return false; }
        let stem = p.file_stem().expect("error").to_str().expect("error");
        return stem.starts_with(video_stem);
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
    fn new_command(&self, path: &PathBuf, subtitle_paths: &Vec<PathBuf>) -> Command {
        let mut command = Command::new("ffmpeg");
        command.args(["-i", path.to_str().expect("Failed to convert to string")]);
        self.add_subtitle_inputs(&mut command, subtitle_paths);
        self.add_args(&mut command);
        self.add_subtitle_args(&mut command, subtitle_paths);
        self.add_output_arg(&mut command, path);

        println!("{:?}", command.get_args());
        command
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

    fn add_subtitle_inputs(&self, command: &mut Command, subtitle_paths: &Vec<PathBuf>) {
        for path in subtitle_paths {
            command.args(["-i", path.to_str().expect("error")]);
            // command.args(["-map", &format!("{}:s:{}", count, count - 1)]);

            // TODO: Detect the language. Hardcoded to english.
            // command.args(["-metadata:s:s:0", "title=\"English\""]);
            // command.args(["-metadata:s:s:0", "language=eng"]);
        }
    }

    fn add_subtitle_args(&self, command: &mut Command, subtitle_paths: &Vec<PathBuf>) {
        // TODO: Handle input labeling more gracefully.
        let mut count = 1;  // Input after video file.
        for path in subtitle_paths {
            command.args(["-map", "0:v:0", "-map", "0:a:0"]);
            command.args(["-map", &format!("{}:s:{}", count, count - 1)]);

            // TODO: Detect the language. Hardcoded to english.
            command.args(["-metadata:s:s:0", "title=\"English\""]);
            command.args(["-metadata:s:s:0", "language=eng"]);
            count += 1;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> { 
    let args: Vec<String> = env::args().collect();
    let mut dir_path = PathBuf::new();
    dir_path.push(&args[1]);
    let config = fs::read_to_string(format!("{}/job.toml", &args[1])).expect("Can't open job.toml");
    let config: Config = toml::from_str(&config)?;

    let paths = get_paths_in_dir(&dir_path).into_iter().filter(|p| is_video_file(p));
    for path in paths {
        let video_subtitle_paths = get_video_subtitle_files(&dir_path, &path);
        let output = config.new_command(&path, &video_subtitle_paths)
            .output()
            .expect("Failed to execute ffmpeg");
        println!("{:?}", output);
    }
    Ok(())
}
