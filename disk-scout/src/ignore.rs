use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct IgnoreConfig {
    pub patterns: Vec<String>,
    pub ignore_file: Option<PathBuf>,
}
