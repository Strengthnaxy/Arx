use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tar::{Archive as Inner, Builder, Header};
use walkdir::WalkDir;

use super::traits::Archive;
use super::{matches_exclude, Format};

pub struct TarArchive {
    path: PathBuf,
    gz: bool,
}

impl TarArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let n = path.to_string_lossy().to_lowercase();
        let gz = n.ends_with(".tar.gz") || n.ends_with(".tgz");
        Ok(Self { path: path.into(), gz })
    }

    pub fn create(
        sources: &[PathBuf],
        output: &Path,
        gz: bool,
        level: u32,
        exclude: &[String],
        verbose: bool,
    ) -> Result<()> {
        let file = File::create(output)
            .with_context(|| format!("не создать {}", output.display()))?;

        if gz {
            let enc = GzEncoder::new(file, Format::from_gz_level(level));
            let mut tar = Builder::new(enc);
            append_sources(&mut tar, sources, exclude, verbose)?;
            tar.finish()?;
        } else {
            let mut tar = Builder::new(file);
            append_sources(&mut tar, sources, exclude, verbose)?;
            tar.finish()?;
        }
        Ok(())
    }

    fn reader(&self) -> Result<Box<dyn Read>> {
        let f = File::open(&self.path)?;
        if self.gz { Ok(Box::new(GzDecoder::new(f))) } else { Ok(Box::new(f)) }
    }
}

fn append_sources<W: Write>(
    tar: &mut Builder<W>,
    sources: &[PathBuf],
    exclude: &[String],
    verbose: bool,
) -> Result<()> {
    for src in sources {
        let parent = src.parent().unwrap_or_else(|| Path::new(""));

        for entry in WalkDir::new(src).min_depth(0).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let rel = path.strip_prefix(parent).unwrap_or(path);
            let name = rel.to_string_lossy().replace('\\', "/");

            if matches_exclude(&name, exclude) {
                if verbose {
                    println!("  skip  {name}");
                }
                continue;
            }

            if path.is_file() {
                let mut f = File::open(path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;

                let mut header = Header::new_gnu();
                header.set_metadata(&entry.metadata()?);
                header.set_path(&name)?;
                header.set_size(buf.len() as u64);
                header.set_cksum();
                tar.append(&header, buf.as_slice())?;

                if verbose {
                    println!("  add   {name}");
                }
            } else if path.is_dir() {
                let mut header = Header::new_gnu();
                header.set_metadata(&entry.metadata()?);
                if !name.is_empty() {
                    header.set_path(format!("{name}/"))?;
                } else {
                    header.set_path("./")?;
                }
                header.set_size(0);
                header.set_entry_type(tar::EntryType::Directory);
                header.set_cksum();
                tar.append(&header, &[][..])?;
            }
        }
    }
    Ok(())
}

impl Archive for TarArchive {
    fn extract(&self, output: &Path) -> Result<()> {
        let mut tar = Inner::new(self.reader()?);
        tar.set_preserve_permissions(true);
        tar.set_overwrite(true);
        tar.unpack(output)?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>> {
        let mut tar = Inner::new(self.reader()?);
        let mut v = Vec::new();
        for e in tar.entries()? {
            v.push(e?.path()?.display().to_string());
        }
        Ok(v)
    }

    fn rename(&self, from: &str, to: &str) -> Result<()> {
        let tmp = self.path.with_extension("tar.tmp");
        let out = File::create(&tmp)?;
        let mut builder: Builder<Box<dyn Write>> = if self.gz {
            Builder::new(Box::new(GzEncoder::new(out, flate2::Compression::default())))
        } else {
            Builder::new(Box::new(out))
        };

        let mut tar = Inner::new(self.reader()?);
        let mut found = false;
        let mut buf = Vec::new();

        for entry in tar.entries()? {
            let mut entry = entry?;
            let name = entry.path()?.display().to_string();
            let new_name = if name == from { found = true; to.into() } else { name };

            buf.clear();
            entry.read_to_end(&mut buf)?;

            let mut header = Header::new_gnu();
            header.set_metadata(&entry.header());
            header.set_path(&new_name)?;
            header.set_size(buf.len() as u64);
            header.set_cksum();
            builder.append(&header, buf.as_slice())?;
        }
        builder.finish()?;

        if !found {
            std::fs::remove_file(&tmp)?;
            bail!("не найдено: {from}");
        }
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
      }
