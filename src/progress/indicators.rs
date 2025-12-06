use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::sync::Arc;

/// Progress manager for the scanning pipeline
pub struct ProgressManager {
    multi: MultiProgress,
    enabled: bool,
}

impl ProgressManager {
    pub fn new(enabled: bool) -> Self {
        let multi = MultiProgress::new();

        if !enabled {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        Self { multi, enabled }
    }

    /// Create a spinner for indeterminate progress (scanning phase)
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb
    }

    /// Create a progress bar for determinate progress (hashing phases)
    pub fn create_progress_bar(&self, total: u64, message: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
                .unwrap()
                .progress_chars("█▓▒░")
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create a bytes progress bar (shows data transferred)
    pub fn create_bytes_bar(&self, total_bytes: u64, message: &str) -> ProgressBar {
        let pb = self.multi.add(ProgressBar::new(total_bytes));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
                .unwrap()
                .progress_chars("█▓▒░")
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Check if progress is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Wrapper to share progress bar across threads
#[derive(Clone)]
pub struct SharedProgress {
    inner: Arc<ProgressBar>,
}

impl SharedProgress {
    pub fn new(pb: ProgressBar) -> Self {
        Self { inner: Arc::new(pb) }
    }

    pub fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    pub fn finish_with_message(&self, msg: &str) {
        self.inner.finish_with_message(msg.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.inner.finish_and_clear();
    }
}
