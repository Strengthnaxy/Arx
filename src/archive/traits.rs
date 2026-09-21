use std::path::Path;
use anyhow::Result;

pub trait Archive {
    fn extract(&self, output: &Path) -> Result<()>;
    fn list(&self) -> Result<Vec<String>>;
    fn rename(&self, from: &str, to: &str) -> Result<()>;
}
