use std::fs;
use std::path::Path;
use anyhow::Result;

use super::{detect, open};

pub fn print(path: &Path) -> Result<()> {
    let fmt = detect(path)?;
    let meta = fs::metadata(path)?;
    let size_kb = meta.len() as f64 / 1024.0;

    let entries = open(path)?.list()?;
    let file_count = entries.iter().filter(|e| !e.ends_with('/')).count();
    let dir_count = entries.len() - file_count;

    println!("файл:      {}", path.display());
    println!("формат:    {}", fmt.name());
    println!("размер:    {:.2} KB ({} bytes)", size_kb, meta.len());
    println!("записей:   {}", entries.len());
    println!("  файлов:  {}", file_count);
    println!("  папок:   {}", dir_count);
    Ok(())
}
