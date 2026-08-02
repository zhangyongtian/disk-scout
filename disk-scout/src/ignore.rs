use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct IgnoreConfig {
    pub patterns: Vec<String>,
    pub ignore_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Default)]
pub struct IgnoreMatcher {
    patterns: Vec<String>,
    ignore_file: Option<PathBuf>,
}

impl IgnoreMatcher {
    pub fn from_config(config: &IgnoreConfig) -> Result<Self, std::io::Error> {
        let mut patterns = Vec::new();
        patterns.extend(config.patterns.iter().cloned());

        if let Some(path) = &config.ignore_file {
            let content = std::fs::read_to_string(path)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                patterns.push(line.to_string());
            }
        }

        Ok(Self {
            patterns,
            ignore_file: config.ignore_file.clone(),
        })
    }

    pub fn is_ignored_path(
        &self,
        absolute_path: &Path,
        relative_path: &str,
        file_name: Option<&str>,
    ) -> bool {
        if let Some(ignore_file) = &self.ignore_file {
            if absolute_path == ignore_file {
                return true;
            }
            if let Some(ignore_name) = ignore_file.file_name().and_then(|s| s.to_str()) {
                if relative_path == ignore_name {
                    return true;
                }
                if file_name == Some(ignore_name) {
                    return true;
                }
            }
        }

        if self.patterns.is_empty() {
            return false;
        }

        for p in &self.patterns {
            if glob_match(p, relative_path) {
                return true;
            }
            if let Some(name) = file_name {
                if glob_match(p, name) {
                    return true;
                }
            }
        }

        false
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let p = pattern.as_bytes();
    let t = text.as_bytes();

    let mut pi = 0usize;
    let mut ti = 0usize;
    let mut star: Option<usize> = None;
    let mut star_match_ti = 0usize;

    while ti < t.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
            continue;
        }

        if pi < p.len() && p[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            star_match_ti = ti;
            continue;
        }

        let Some(star_pi) = star else {
            return false;
        };

        star_match_ti += 1;
        ti = star_match_ti;
        pi = star_pi + 1;
    }

    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }

    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_star_and_question() {
        assert!(glob_match("*.log", "a.log"));
        assert!(glob_match("a?c", "abc"));
        assert!(glob_match("a*c", "abbbbbc"));
        assert!(!glob_match("a?c", "ac"));
    }

    #[test]
    fn matcher_checks_full_path_and_file_name() {
        let m = IgnoreMatcher {
            patterns: vec!["*.tmp".to_string(), "target/*".to_string()],
            ignore_file: None,
        };
        assert!(m.is_ignored_path(Path::new("foo.tmp"), "foo.tmp", Some("foo.tmp")));
        assert!(m.is_ignored_path(
            Path::new("target/debug/a"),
            "target/debug/a",
            Some("a")
        ));
        assert!(!m.is_ignored_path(
            Path::new("src/main.rs"),
            "src/main.rs",
            Some("main.rs")
        ));
    }
}
