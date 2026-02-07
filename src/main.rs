use clap::{Parser, Subcommand};
use ffgo::Config;
use ffgo::file_constants::{JOB_CONFIG_FILE_NAME, READY_FILE_NAME};
use ffgo::jobs;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify::event::CreateKind;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: Option<String>,

    #[arg(short, long, default_value_t = 1)]
    jobs: usize,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Watch {
        path: PathBuf 
    }
}

async fn go(queue: &jobs::Queue, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut config_path = PathBuf::new();
    config_path.push(path);
    config_path.push(*JOB_CONFIG_FILE_NAME);
    let config = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config)?;
    queue.push_directory(path, &config).await;
    Ok(())
}

async fn watch(queue: &jobs::Queue, path: &Path) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path, RecursiveMode::Recursive)?;
    for res in rx {
        println!("{:?}", res);
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Create(CreateKind::File) => {
                        let Some(path) = event.paths.first() else { continue; };
                        let Some(file_stem) = path.as_path().file_stem() else {
                            println!("Failed to get file name of {:?}", path);
                            continue;
                        };

                        if file_stem == *READY_FILE_NAME {
                            let Some(parent_path) = path.parent() else { continue; };
                            let result = go(queue, parent_path).await;
                            match result {
                                Err(e) => { println!("{}", e); }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => { println!("File system watch error: {}", e); }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> { 
    let args = Args::parse();
    let queue = jobs::Queue::new(args.jobs);
    match &args.command {
        Some(Command::Watch { path }) => {
            match watch(&queue, path).await {
                Err(e) => {
                    println!("Failed to watch directory due to error: {}", e);
                },
                _ => {},
            }
        }

        None => {
            let Some(path) = &args.path else {
                return Err("Must specify a PATH".into());
            };
            let mut dir_path = PathBuf::new();
            dir_path.push(path);
            let result = go(&queue, dir_path.as_path()).await;
            match result {
                Err(e) => println!("{}", e),
                _ => {}
            }
        }
    }
    Ok(())
}
