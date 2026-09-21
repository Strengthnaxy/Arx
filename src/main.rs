mod archive;
mod cli;
mod progress;

use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;

use cli::Cli;

fn main() -> Result<()> {
    let c = Cli::parse();

    // arx -u <archive>
    if c.unpack {
        let input = c.input.first().ok_or_else(|| anyhow::anyhow!("укажи архив"))?;
        return archive::extract(input, &c.out_dir, c.quiet);
    }

    // arx -l <archive>
    if c.list {
        let input = c.input.first().ok_or_else(|| anyhow::anyhow!("укажи архив"))?;
        return archive::list(input);
    }

    // arx -i <archive>
    if c.info {
        let input = c.input.first().ok_or_else(|| anyhow::anyhow!("укажи архив"))?;
        return archive::info(input);
    }

    // arx -r <archive> <from> <to>
    if let Some(args) = &c.rename {
        if args.len() != 3 {
            bail!("использование: arx -r <archive> <from> <to>");
        }
        return archive::rename(
            std::path::Path::new(&args[0]),
            &args[1],
            &args[2],
        );
    }

    // arx -p <src...> <out>   (авто-формат по имени)
    // arx -z <src...> <out.zip>
    // arx -t <src...> <out.tar>
    // arx -T <src...> <out.tar.gz>
    if c.pack || c.zip || c.tar || c.tar_gz {
        if c.input.is_empty() {
            bail!("укажи источник: arx -p <src> <out.zip>");
        }

        // Последний позиционный аргумент — выходной архив
        let (sources, output) = split_out(&c.input);
        let output = output.ok_or_else(|| anyhow::anyhow!("укажи имя архива"))?;

        let force = if c.zip {
            Some(archive::Format::Zip)
        } else if c.tar_gz {
            Some(archive::Format::TarGz)
        } else if c.tar {
            Some(archive::Format::Tar)
        } else {
            None // -p → авто по имени
        };

        return archive::create(
            &sources,
            &output,
            force,
            c.level,
            &c.exclude,
            c.verbose,
            c.quiet,
        );
    }

    use clap::CommandFactory;
    Cli::command().print_help()?;
    println!();
    Ok(())
}

/// Разбивает позиционные аргументы на (sources, output).
/// Если последний аргумент имеет расширение архива — это output.
fn split_out(args: &[PathBuf]) -> (Vec<PathBuf>, Option<PathBuf>) {
    if args.is_empty() {
        return (vec![], None);
    }
    let last = args.last().unwrap();
    if archive::detect(last).is_ok() && args.len() >= 2 {
        (args[..args.len() - 1].to_vec(), Some(last.clone()))
    } else {
        (args.to_vec(), None)
    }
}
