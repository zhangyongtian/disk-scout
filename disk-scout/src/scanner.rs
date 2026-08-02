use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::{ignore::IgnoreMatcher, plan::ScanPlan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SizedPath {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ScanStats {
    pub files_seen: u64,
    pub dirs_seen: u64,
    pub bytes_total: u64,
    pub errors_total: u64,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct ScanErrorSample {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct ScanErrors {
    pub total: u64,
    pub samples: Vec<ScanErrorSample>,
}

#[derive(Clone, Debug)]
pub struct ScanResult {
    pub root: PathBuf,
    pub stats: ScanStats,
    pub top_files: Vec<SizedPath>,
    pub top_dirs: Vec<SizedPath>,
    pub errors: ScanErrors,
}

#[derive(Debug)]
pub struct ScanInitError {
    pub root: PathBuf,
    pub message: String,
}

pub fn scan(plan: &ScanPlan) -> Result<ScanResult, ScanInitError> {
    let started_at = Instant::now();

    let root_meta = std::fs::symlink_metadata(&plan.root).map_err(|e| ScanInitError {
        root: plan.root.clone(),
        message: e.to_string(),
    })?;

    if root_meta.file_type().is_symlink() {
        return Err(ScanInitError {
            root: plan.root.clone(),
            message: "root path is a symlink (not followed by default)".to_string(),
        });
    }

    let mut state = ScanState::new(plan.top_files, plan.top_dirs, plan.min_size);
    let ignore = IgnoreMatcher::from_config(&plan.ignore).map_err(|e| ScanInitError {
        root: plan.root.clone(),
        message: format!("failed to read ignore rules: {e}"),
    })?;

    let bytes_total = if root_meta.is_dir() {
        scan_dir(&plan.root, &plan.root, &ignore, &mut state)
    } else if root_meta.is_file() {
        let size = root_meta.len();
        state.stats.files_seen += 1;
        state.consider_file(plan.root.clone(), size);
        size
    } else {
        0
    };

    state.stats.bytes_total = bytes_total;
    state.stats.duration = started_at.elapsed();
    state.stats.errors_total = state.errors.total;

    Ok(ScanResult {
        root: plan.root.clone(),
        stats: state.stats,
        top_files: state.top_files.into_sorted_desc(),
        top_dirs: state.top_dirs.into_sorted_desc(),
        errors: state.errors,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeapItem {
    size: u64,
    path: PathBuf,
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .cmp(&other.size)
            .then_with(|| self.path.cmp(&other.path))
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
struct TopN {
    capacity: usize,
    heap: BinaryHeap<std::cmp::Reverse<HeapItem>>,
}

impl TopN {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    fn consider(&mut self, path: PathBuf, size: u64) {
        if self.capacity == 0 {
            return;
        }

        let item = HeapItem { size, path };

        if self.heap.len() < self.capacity {
            self.heap.push(std::cmp::Reverse(item));
            return;
        }

        let Some(min) = self.heap.peek() else {
            self.heap.push(std::cmp::Reverse(item));
            return;
        };

        if item.size > min.0.size {
            self.heap.pop();
            self.heap.push(std::cmp::Reverse(item));
        }
    }

    fn into_sorted_desc(self) -> Vec<SizedPath> {
        let mut items: Vec<SizedPath> = self
            .heap
            .into_iter()
            .map(|x| SizedPath {
                path: x.0.path,
                size: x.0.size,
            })
            .collect();
        items.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.path.cmp(&b.path)));
        items
    }
}

struct ScanState {
    stats: ScanStats,
    top_files: TopN,
    top_dirs: TopN,
    min_file_size_for_top: u64,
    errors: ScanErrors,
}

impl ScanState {
    fn new(top_files: usize, top_dirs: usize, min_file_size_for_top: u64) -> Self {
        Self {
            stats: ScanStats::default(),
            top_files: TopN::new(top_files),
            top_dirs: TopN::new(top_dirs),
            min_file_size_for_top,
            errors: ScanErrors::default(),
        }
    }

    fn record_error(&mut self, path: PathBuf, message: String) {
        self.errors.total += 1;
        if self.errors.samples.len() < 20 {
            self.errors.samples.push(ScanErrorSample { path, message });
        }
    }

    fn consider_file(&mut self, path: PathBuf, size: u64) {
        if size >= self.min_file_size_for_top {
            self.top_files.consider(path, size);
        }
    }

    fn consider_dir(&mut self, path: PathBuf, size: u64) {
        self.top_dirs.consider(path, size);
    }
}

fn scan_dir(root: &Path, path: &Path, ignore: &IgnoreMatcher, state: &mut ScanState) -> u64 {
    state.stats.dirs_seen += 1;

    let mut sum = 0u64;

    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            state.record_error(path.to_path_buf(), e.to_string());
            return 0;
        }
    };

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                state.record_error(path.to_path_buf(), e.to_string());
                continue;
            }
        };

        let child_path = entry.path();
        let child_rel = match child_path.strip_prefix(root) {
            Ok(p) => normalize_rel_path(p),
            Err(_) => child_path.to_string_lossy().to_string(),
        };
        let child_name = child_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        let meta = match std::fs::symlink_metadata(&child_path) {
            Ok(m) => m,
            Err(e) => {
                state.record_error(child_path, e.to_string());
                continue;
            }
        };

        let ft = meta.file_type();
        if ft.is_symlink() {
            continue;
        }

        if meta.is_dir() {
            if ignore.is_ignored_path(&child_rel, child_name.as_deref()) {
                continue;
            }
            sum = sum.saturating_add(scan_dir(root, &child_path, ignore, state));
            continue;
        }

        if meta.is_file() {
            if ignore.is_ignored_path(&child_rel, child_name.as_deref()) {
                continue;
            }
            state.stats.files_seen += 1;
            let size = meta.len();
            sum = sum.saturating_add(size);
            state.consider_file(child_path, size);
            continue;
        }
    }

    state.consider_dir(path.to_path_buf(), sum);
    sum
}

fn normalize_rel_path(path: &Path) -> String {
    let mut out = String::new();
    for c in path.components() {
        let s = c.as_os_str().to_string_lossy();
        if s.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str(&s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_tmp_root(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("disk_scout_test_{name}_{t}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn aggregates_dir_sizes_and_topn_files() {
        let root = mk_tmp_root("agg");
        let d1 = root.join("d1");
        std::fs::create_dir_all(&d1).unwrap();
        std::fs::write(d1.join("a.bin"), vec![0u8; 10]).unwrap();
        std::fs::write(d1.join("b.bin"), vec![0u8; 30]).unwrap();
        std::fs::write(root.join("c.bin"), vec![0u8; 20]).unwrap();

        let plan = ScanPlan {
            root: root.clone(),
            top_files: 2,
            top_dirs: 10,
            min_size: 0,
            format: crate::output::OutputFormat::Text,
            ignore: crate::ignore::IgnoreConfig::default(),
        };

        let result = scan(&plan).unwrap();

        assert_eq!(result.stats.files_seen, 3);
        assert!(result.stats.dirs_seen >= 2);
        assert_eq!(result.stats.bytes_total, 60);
        assert_eq!(result.top_files.len(), 2);
        assert_eq!(result.top_files[0].size, 30);
        assert_eq!(result.top_files[1].size, 20);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn applies_min_size_and_ignore_rules() {
        let root = mk_tmp_root("filter");
        let keep = root.join("keep");
        let skip = root.join("skip");
        std::fs::create_dir_all(&keep).unwrap();
        std::fs::create_dir_all(&skip).unwrap();

        std::fs::write(keep.join("small.bin"), vec![0u8; 10]).unwrap();
        std::fs::write(keep.join("big.bin"), vec![0u8; 2048]).unwrap();
        std::fs::write(skip.join("ignored.bin"), vec![0u8; 4096]).unwrap();

        let ignore_file = root.join(".disk-scout-ignore");
        std::fs::write(&ignore_file, "# comment\nskip/*\n").unwrap();

        let plan = ScanPlan {
            root: root.clone(),
            top_files: 10,
            top_dirs: 10,
            min_size: 1024,
            format: crate::output::OutputFormat::Text,
            ignore: crate::ignore::IgnoreConfig {
                patterns: vec!["*.tmp".to_string()],
                ignore_file: Some(ignore_file),
            },
        };

        let result = scan(&plan).unwrap();

        assert_eq!(result.stats.files_seen, 2);
        assert_eq!(result.stats.bytes_total, 10 + 2048);
        assert_eq!(result.top_files.len(), 1);
        assert!(result.top_files[0].path.ends_with("big.bin"));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn does_not_follow_symlink() {
        use std::os::unix::fs::symlink;

        let root = mk_tmp_root("symlink");
        let real = root.join("real");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join("x.bin"), vec![0u8; 50]).unwrap();

        let link = root.join("link");
        symlink(&real, &link).unwrap();

        let plan = ScanPlan {
            root: root.clone(),
            top_files: 10,
            top_dirs: 10,
            min_size: 0,
            format: crate::output::OutputFormat::Text,
            ignore: crate::ignore::IgnoreConfig::default(),
        };

        let result = scan(&plan).unwrap();
        assert_eq!(result.stats.bytes_total, 50);
        assert_eq!(result.stats.files_seen, 1);

        std::fs::remove_dir_all(&root).unwrap();
    }
}
