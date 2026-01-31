use clap::{Parser, Subcommand};
use ffgo::Config;
use ffgo::jobs;
use ffgo::path_strings::PathStrings;
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
    config_path.push("job.toml");
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
                        if path.as_path().file_stem_str().unwrap_or("") == ".ready" {
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
            watch(&queue, path).await;
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
