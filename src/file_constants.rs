use once_cell::sync::Lazy;
use std::ffi::OsStr;

pub static JOB_CONFIG_FILE_NAME: Lazy<&'static OsStr> = Lazy::new(|| {
  OsStr::new("job.toml")
});

pub static READY_FILE_NAME: Lazy<&'static OsStr> = Lazy::new(|| {
  OsStr::new(".ready")
});
