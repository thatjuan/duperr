use std::path::Path;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs::Metadata;

pub struct FileFilter {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub extensions: Option<Vec<String>>,
    pub exclude_patterns: GlobSet,
    pub include_hidden: bool,
    pub skip_empty: bool,
}

impl FileFilter {
    pub fn new(
        min_size: Option<u64>,
        max_size: Option<u64>,
        extensions: Option<Vec<String>>,
        exclude: &[String],
        include_hidden: bool,
        skip_empty: bool,
    ) -> Result<Self, globset::Error> {
        let mut builder = GlobSetBuilder::new();
        for pattern in exclude {
            builder.add(Glob::new(pattern)?);
        }

        Ok(Self {
            min_size,
            max_size,
            extensions: extensions.map(|exts| {
                exts.into_iter().map(|e| e.to_lowercase()).collect()
            }),
            exclude_patterns: builder.build()?,
            include_hidden,
            skip_empty,
        })
    }

    /// Check if file should be included
    pub fn should_include(&self, path: &Path, metadata: &Metadata) -> bool {
        let size = metadata.len();

        // Skip empty files if configured
        if self.skip_empty && size == 0 {
            return false;
        }

        // Check size bounds
        if let Some(min) = self.min_size {
            if size < min { return false; }
        }
        if let Some(max) = self.max_size {
            if size > max { return false; }
        }

        // Check hidden files
        if !self.include_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') { return false; }
            }
        }

        // Check extension filter
        if let Some(ref exts) = self.extensions {
            let file_ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            match file_ext {
                Some(ext) if exts.contains(&ext) => {}
                _ => return false,
            }
        }

        // Check exclude patterns
        if self.exclude_patterns.is_match(path) {
            return false;
        }

        true
    }

    /// Check if this is a regular file (not FIFO, socket, device, etc.)
    pub fn is_regular_file(metadata: &Metadata) -> bool {
        metadata.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_extension_filter() {
        let filter = FileFilter::new(
            None, None,
            Some(vec!["txt".to_string(), "md".to_string()]),
            &[], false, false, // skip_empty = false
        ).unwrap();

        let dir = tempdir().unwrap();
        let txt_path = dir.path().join("test.txt");
        let jpg_path = dir.path().join("test.jpg");

        std::fs::write(&txt_path, "content").unwrap();
        std::fs::write(&jpg_path, "content").unwrap();

        let txt_meta = std::fs::metadata(&txt_path).unwrap();
        let jpg_meta = std::fs::metadata(&jpg_path).unwrap();

        assert!(filter.should_include(&txt_path, &txt_meta));
        assert!(!filter.should_include(&jpg_path, &jpg_meta));
    }

    #[test]
    fn test_size_filter() {
        let filter = FileFilter::new(
            Some(100), Some(1000),
            None, &[], false, true,
        ).unwrap();

        let dir = tempdir().unwrap();
        let small = dir.path().join("small.txt");
        let medium = dir.path().join("medium.txt");
        let large = dir.path().join("large.txt");

        std::fs::write(&small, "x".repeat(50)).unwrap();
        std::fs::write(&medium, "x".repeat(500)).unwrap();
        std::fs::write(&large, "x".repeat(2000)).unwrap();

        let small_meta = std::fs::metadata(&small).unwrap();
        let medium_meta = std::fs::metadata(&medium).unwrap();
        let large_meta = std::fs::metadata(&large).unwrap();

        assert!(!filter.should_include(&small, &small_meta)); // too small
        assert!(filter.should_include(&medium, &medium_meta)); // just right
        assert!(!filter.should_include(&large, &large_meta)); // too large
    }

    #[test]
    fn test_hidden_file_filter() {
        let filter = FileFilter::new(None, None, None, &[], false, false).unwrap();
        let filter_include = FileFilter::new(None, None, None, &[], true, false).unwrap();

        let dir = tempdir().unwrap();
        let hidden = dir.path().join(".hidden");
        std::fs::write(&hidden, "content").unwrap();
        let meta = std::fs::metadata(&hidden).unwrap();

        assert!(!filter.should_include(&hidden, &meta));
        assert!(filter_include.should_include(&hidden, &meta));
    }

    #[test]
    fn test_empty_file_filter() {
        let filter_skip = FileFilter::new(None, None, None, &[], false, true).unwrap();
        let filter_include = FileFilter::new(None, None, None, &[], false, false).unwrap();

        let dir = tempdir().unwrap();
        let empty = dir.path().join("empty.txt");
        File::create(&empty).unwrap();
        let meta = std::fs::metadata(&empty).unwrap();

        assert!(!filter_skip.should_include(&empty, &meta));
        assert!(filter_include.should_include(&empty, &meta));
    }

    #[test]
    fn test_exclude_pattern() {
        let filter = FileFilter::new(
            None, None, None,
            &["*.tmp".to_string(), "cache/*".to_string()],
            false, false,
        ).unwrap();

        let dir = tempdir().unwrap();
        let tmp_file = dir.path().join("temp.tmp");
        let txt_file = dir.path().join("file.txt");

        std::fs::write(&tmp_file, "content").unwrap();
        std::fs::write(&txt_file, "content").unwrap();

        let tmp_meta = std::fs::metadata(&tmp_file).unwrap();
        let txt_meta = std::fs::metadata(&txt_file).unwrap();

        assert!(!filter.should_include(&tmp_file, &tmp_meta));
        assert!(filter.should_include(&txt_file, &txt_meta));
    }
}
