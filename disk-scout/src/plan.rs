use std::path::PathBuf;

use crate::ignore::IgnoreConfig;

#[derive(Clone, Debug)]
pub struct ScanPlan {
    pub root: PathBuf,
    pub top_files: usize,
    pub top_dirs: usize,
    pub min_size: u64,
    pub ignore: IgnoreConfig,
}
