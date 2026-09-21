pub mod traits;
mod info;
mod tar;
mod zip;

use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};
use traits::Archive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Zip,
    Tar,
    TarGz,
}

impl Format {
    pub fn name(&self) -> &'static str {
        match self {
            Format::Zip => "zip",
            Format::Tar => "tar",
            Format::TarGz => "tar.gz",
        }
    }

    pub fn from_zip_level(level: u32) -> zip::CompressionMethod {
        if level == 0 {
            zip::CompressionMethod::Stored
        } else {
            zip::CompressionMethod::Deflated
        }
    }

    pub fn from_gz_level(level: u32) -> flate2::Compression {
        flate2::Compression::new(level)
    }
}

/// Определяет формат по расширению
pub fn detect(path: &Path) -> Result<Format> {
    let name = path.to_string_lossy().to_lowercase();
    if name.ends_with(".zip") {
        Ok(Format::Zip)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok(Format::TarGz)
    } else if name.ends_with(".tar") {
        Ok(Format::Tar)
    } else {
        bail!("неподдерживаемый формат: {name}")
    }
}

fn open(path: &Path) -> Result<Box<dyn Archive>> {
    if !path.exists() {
        bail!("файл не найден: {}", path.display());
    }
    Ok(match detect(path)? {
        Format::Zip => Box::new(zip::ZipArchive::open(path)?),
        Format::Tar | Format::TarGz => Box::new(tar::TarArchive::open(path)?),
    })
}

/// Распаковка
pub fn extract(input: &Path, output: &Path, quiet: bool) -> Result<()> {
    std::fs::create_dir_all(output)
        .with_context(|| format!("не создать {}", output.display()))?;

    let archive = open(input)?;
    let entries = archive.list()?;
    let bar = if quiet {
        None
    } else {
        Some(crate::progress::Progress::new("unpack", entries.len() as u64))
    };

    archive.extract(output)?;

    if let Some(b) = bar {
        b.finish();
    }
    println!("✓ распаковано в {}", output.display());
    Ok(())
}

/// Упаковка: sources — список путей, output — имя архива.
#[allow(clippy::too_many_arguments)]
pub fn create(
    sources: &[PathBuf],
    output: &Path,
    force: Option<Format>,
    level: u32,
    exclude: &[String],
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    for s in sources {
        if !s.exists() {
            bail!("источник не найден: {}", s.display());
        }
    }

    let fmt = match force {
        Some(f) => f,
        None => detect(output)?,
    };

    if !quiet {
        println!(
            "→ упаковка {} источник(ов) в {} (формат: {}, уровень: {})",
            sources.len(),
            output.display(),
            fmt.name(),
            level
        );
    }

    match fmt {
        Format::Zip => zip::ZipArchive::create(sources, output, level, exclude, verbose)?,
        Format::Tar => tar::TarArchive::create(sources, output, false, level, exclude, verbose)?,
        Format::TarGz => tar::TarArchive::create(sources, output, true, level, exclude, verbose)?,
    }

    println!("✓ создан {} ({})", output.display(), fmt.name());
    Ok(())
}

pub fn list(input: &Path) -> Result<()> {
    for name in open(input)?.list()? {
        println!("{name}");
    }
    Ok(())
}

pub fn info(input: &Path) -> Result<()> {
    info::print(input)
}

pub fn rename(input: &Path, from: &str, to: &str) -> Result<()> {
    open(input)?.rename(from, to)?;
    println!("✓ {from} → {to}");
    Ok(())
}

/// Проверяет, матчится ли путь под паттерн исключения.
/// Паттерн — подстрока или simple-glob (*.log, target/*).
pub fn matches_exclude(path: &str, patterns: &[String]) -> bool {
    for p in patterns {
        if p.is_empty() {
            continue;
        }
        // простой глоб: '*' = любые символы
        if simple_glob(p, path) {
            return true;
        }
        // подстрока
        if path.contains(p.as_str()) {
            return true;
        }
    }
    false
}

/// Простейший glob только с '*'.
fn simple_glob(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !text[pos..].starts_with(part) {
                return false;
            }
            pos += part.len();
        } else if i == parts.len() - 1 {
            return text[pos..].ends_with(part);
        } else if let Some(found) = text[pos..].find(part) {
            pos += found + part.len();
        } else {
            return false;
        }
    }
    true
      }
