use std::clone::Clone;
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
  pub fn new() -> Queue {
    let (tx, rx) = mpsc::channel::<Job>(100);
    let queue = Queue {
      tx: tx,
    };
    queue.start(rx);
    queue
  }

  fn start(&self, mut rx: mpsc::Receiver<Job>) {
    tokio::spawn(async move {
      let semaphore = Arc::new(Semaphore::new(2));
      while let Some(job) = rx.recv().await {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        tokio::task::spawn_blocking(move || {
          let _permit = permit;
          println!("Processing {:?}", job.input_files);
          let output = job.config.new_command(&job.input_files)
              .output()
              .expect("Failed to execute ffmpeg");
          println!("{:?}", output);
          if let Some(tx) = job.tx {
            tx.send(true).unwrap();
          }
        });
      }
    });
  }

  pub async fn push(&self, mut job: Job) -> oneshot::Receiver<bool> {
    let (tx, rx) = oneshot::channel();
    job.tx = Some(tx);
    self.tx.send(job).await.unwrap();
    rx
  }

  pub async fn push_directory(&self, path: &Path, config: &Config) {
    let mut job_handles = Vec::<oneshot::Receiver<bool>>::new();
    for (_, mut files) in file_utils::get_jobs(path) {
        files.sort();
        println!("pusing: {:?}", files);
        job_handles.push(self.push(Job::new(config.clone(), files)).await);
    }

    for handle in job_handles {
        let result = handle.await.unwrap();
        println!("Result: {}", result);
    }
  }
}