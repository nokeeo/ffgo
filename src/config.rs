use isolang::Language;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command; 
use crate::path_strings::PathStrings;

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub extension: String,
    pub directory: String
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub args: Vec<String>,
    pub output: OutputConfig,
    pub auto_map_external_subtitle_inputs: Option<bool>
}

impl Config {
    pub fn auto_map_external_subtitle_inputs(&self) -> bool {
        match self.auto_map_external_subtitle_inputs {
            Some(value) => value,
            _ => false,
        }
    }

    pub fn new_command(&self, files: &Vec<PathBuf>) -> Command {
        let mut command = Command::new("ffmpeg");
        self.add_input_files(&mut command, files);
        self.add_args(&mut command, files);
        self.add_output_arg(&mut command, files.first().unwrap());

        println!("{:?}", command.get_args());
        command
    }

    fn add_input_files(&self, command: &mut Command, files: &Vec<PathBuf>) {
        for path in files {
            command.args(["-i", path.to_str().unwrap()]);
        }
    }

    fn add_args(&self, command: &mut Command, files: &Vec<PathBuf>) {
        for arg in self.args.iter() {
            for part in arg.split(' ').collect::<Vec<&str>>() {
                command.arg(part);
            }
        }

        if self.auto_map_external_subtitle_inputs() {
            let subtitle_paths = files.iter()
                .enumerate()
                .filter(|(_, p)| is_subtitle_file(p));
            let mut stream_i = 0;
            for (i, path) in subtitle_paths {
                command.args(["-map", &format!("{}:s:{}", i, stream_i)]);
                let code = get_subtitle_language_code(path);
                if let Ok(code) = code {
                    command.arg(&format!("-metadata:s:s:{}", stream_i));
                    command.arg(&format!("language={}", code));
                }
                stream_i += 1;
            }
        }
    }

    fn add_output_arg(&self, command: &mut Command, path: &Path) { 
        let Some(filename) = path.file_stem_str() else {
            panic!("Failed to get filename of path: {:?}", path);
        };
        command.arg(format!("{}/{}.{}", self.output.directory, filename, self.output.extension));
    }
}

fn is_subtitle_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        return match extension {
            "srt" | "vtt" | "ssa" | "scc" | "stl" => true,
            _ => false
        };
    }
    false
}

fn get_subtitle_language_code(path: &Path) -> Result<&'static str, String> {
    let Some(stem) = path.file_stem_str() else {
        return Err(format!("Failed to parse stem: {:?}", path));
    };

    let Some(last) = stem.split("_").last() else {
        return Err(format!("Failed to extract language code in {:?}", stem));
    };

    let lang1 = Language::from_639_1(last).ok_or("Failed to parse subtitle part 1")?;
    Ok(lang1.to_639_3())
}


