use std::clone::Clone;
use std::io;
use std::path::PathBuf;
use std::path::Path; 
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;
use crate::jobs::file_utils;

use crate::Config;

pub struct Job {
  config: Config,
  input_files: Vec<PathBuf>,
  tx: Option<oneshot::Sender<bool>>
}

pub struct Queue {
  tx: mpsc::Sender<Job>,
}

impl Job {
  pub fn new(config: Config, input_files: Vec<PathBuf>) -> Job {
    Job {
      config: config,
      input_files: input_files,
      tx: None,
    }
  }
}

impl Queue {
  pub fn new(job_count: usize) -> Queue {
    let (tx, rx) = mpsc::channel::<Job>(100);
    let queue = Queue {
      tx: tx,
    };
    queue.start(job_count, rx);
    queue
  }

  fn start(&self, job_count: usize, mut rx: mpsc::Receiver<Job>) {
    tokio::spawn(async move {
      let semaphore = Arc::new(Semaphore::new(job_count));
      while let Some(job) = rx.recv().await {
        let permit = match semaphore.clone().acquire_owned().await {
          Ok(p) => p,
          Err(e) => {
            println!("Failed to acquire semaphore with error {:?}", e);
            continue;
          }
        };

        tokio::task::spawn_blocking(move || {
          let _permit = permit;
          println!("Processing {:?}", job.input_files);
          let mut child = match job.config.new_command(&job.input_files).spawn() {
            Ok(c) => c,
            Err(e) => {
              println!("Failed to spawn ffmpeg child process: {:?}", e);
              return;
            }
          };

          child.wait().expect("failed to wait on child");
          if let Some(tx) = job.tx {
            match tx.send(true) {
              Err(e) => {
                println!("Failed to send result: {:?}", e);
              }
              _ => {}
            }
          }
        });
      }
    });
  }

  pub async fn push(&self, mut job: Job) -> Result<oneshot::Receiver<bool>, Box<dyn std::error::Error>> {
    let (tx, rx) = oneshot::channel();
    job.tx = Some(tx);
    self.tx.send(job).await?;
    Ok(rx)
  }

  pub async fn push_directory(&self, path: &Path, config: &Config) -> io::Result<()> {
    let mut job_handles = Vec::<oneshot::Receiver<bool>>::new();
    for (_, mut files) in file_utils::get_jobs(path)? {
        files.sort();
        println!("Scheduling: {:?}", files);
        match self.push(Job::new(config.clone(), files)).await {
          Ok(handle) => job_handles.push(handle),
          Err(e) => println!("Failed to schedule job: {:?}", e),
        }
    }

    for handle in job_handles {
        match handle.await {
          Ok(result) => println!("Result: {}", result),
          Err(e) => println!("Failed to retrieve results: {:?}", e),
        }
    }
    Ok(())
  }
}