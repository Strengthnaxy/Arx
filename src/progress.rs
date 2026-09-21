use indicatif::{ProgressBar, ProgressStyle};

pub struct Progress(ProgressBar);

impl Progress {
    pub fn new(msg: &str, total: u64) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template("{msg} [{bar:40}] {pos}/{len}")
                .unwrap()
                .progress_chars("=>-"),
        );
        bar.set_message(msg.to_string());
        Self(bar)
    }
    pub fn finish(&self) { self.0.finish_and_clear(); }
}
