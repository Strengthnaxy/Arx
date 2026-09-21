use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use walkdir::WalkDir;
use zip::{ZipArchive as Inner, ZipWriter, write::FileOptions};

use super::traits::Archive;
use super::{matches_exclude, Format};

pub struct ZipArchive {
    path: PathBuf,
    inner: Inner<File>,
}

impl ZipArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("не открыть {}", path.display()))?;
        let inner = Inner::new(file)
            .with_context(|| format!("повреждённый zip: {}", path.display()))?;
        Ok(Self { path: path.to_path_buf(), inner })
    }

    pub fn create(
        sources: &[PathBuf],
        output: &Path,
        level: u32,
        exclude: &[String],
        verbose: bool,
    ) -> Result<()> {
        let file = File::create(output)?;
        let mut zip = ZipWriter::new(file);

        let opts: FileOptions<()> = FileOptions::default()
            .compression_method(Format::from_zip_level(level));

        for src in sources {
            add_path(&mut zip, src, src, opts, exclude, verbose)?;
        }
        zip.finish()?;
        Ok(())
    }
}

fn add_path(
    zip: &mut ZipWriter<File>,
    base: &Path,
    path: &Path,
    opts: FileOptions<()>,
    exclude: &[String],
    verbose: bool,
) -> Result<()> {
    // имя внутри архива — относительно родителя base
    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let rel = path.strip_prefix(parent).unwrap_or(path);
    let name = rel.to_string_lossy().replace('\\', "/");

    if !name.is_empty() && matches_exclude(&name, exclude) {
        if verbose {
            println!("  skip  {name}");
        }
        return Ok(());
    }

    if path.is_file() {
        zip.start_file(&name, opts)?;
        let mut f = File::open(path)?;
        io::copy(&mut f, zip)?;
        if verbose {
            println!("  add   {name}");
        }
    } else if path.is_dir() {
        if !name.is_empty() {
            zip.add_directory(format!("{name}/"), opts)?;
        }
        for entry in WalkDir::new(path).min_depth(1).into_iter().filter_map(|e| e.ok()) {
            let sub = entry.path();
            let sub_rel = sub.strip_prefix(parent).unwrap_or(sub);
            let sub_name = sub_rel.to_string_lossy().replace('\\', "/");
            if matches_exclude(&sub_name, exclude) {
                if verbose {
                    println!("  skip  {sub_name}");
                }
                continue;
            }
            if sub.is_file() {
                zip.start_file(&sub_name, opts)?;
                let mut f = File::open(sub)?;
                io::copy(&mut f, zip)?;
                if verbose {
                    println!("  add   {sub_name}");
                }
            } else if sub.is_dir() && !sub_name.is_empty() {
                zip.add_directory(format!("{sub_name}/"), opts)?;
            }
        }
    }
    Ok(())
}

impl Archive for ZipArchive {
    fn extract(&self, output: &Path) -> Result<()> {
        let file = File::open(&self.path)?;
        let mut zip = Inner::new(file)?;

        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let outpath = match entry.enclosed_name() {
                Some(p) => output.join(p),
                None => bail!("небезопасный путь: {}", entry.name()),
            };
            if entry.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut out = File::create(&outpath)?;
                io::copy(&mut entry, &mut out)?;
            }
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>> {
        let file = File::open(&self.path)?;
        let zip = Inner::new(file)?;
        Ok(zip.file_names().map(String::from).collect())
    }

    fn rename(&self, from: &str, to: &str) -> Result<()> {
        let file = File::open(&self.path)?;
        let mut zip = Inner::new(file)?;

        let tmp = self.path.with_extension("zip.tmp");
        let out_file = File::create(&tmp)?;
        let mut writer = ZipWriter::new(out_file);
        let opts: FileOptions<()> = FileOptions::default();

        let mut found = false;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let name = entry.name().to_string();
            let new_name = if name == from { found = true; to.into() } else { name };

            if entry.is_dir() {
                writer.add_directory(new_name, opts)?;
            } else {
                writer.start_file(new_name, opts)?;
                io::copy(&mut entry, &mut writer)?;
            }
        }
        writer.finish()?;

        if !found {
            std::fs::remove_file(&tmp)?;
            bail!("не найдено: {from}");
        }
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}
