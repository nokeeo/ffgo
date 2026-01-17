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
    fn new_command(&self, path: &PathBuf) -> Command {
        let mut command = Command::new("ffmpeg");
        command.args(["-i", path.to_str().expect("Failed to convert to string")]);
        self.add_args(&mut command);
        self.add_output_arg(&mut command, path);
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
}

fn main() -> Result<(), Box<dyn Error>> { 
    let args: Vec<String> = env::args().collect();
    let config = fs::read_to_string(format!("{}/job.toml", &args[1])).expect("Can't open job.toml");
    let config: Config = toml::from_str(&config)?;

    let paths = std::fs::read_dir(&args[1])?.filter_map(|entry| {
        return match entry {
            Ok(entry) => if is_video_file(&entry.path()) { Some(entry.path()) } else { None },
            _ => None
        };
    });

    for path in paths {
        let output = config.new_command(&path)
            .output()
            .expect("Failed to execute ffmpeg");
        println!("{:?}", output);
    }
    Ok(())
}
