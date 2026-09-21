use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "arx",
    version,
    about = "arx — zip, tar и tar.gz в одной команде",
    long_about = "\
arx — единый инструмент для работы с архивами.
Поддерживает zip, tar и tar.gz.
Формат определяется по расширению автоматически.\n
Примеры:
  arx -u backup.zip
  arx -u backup.tar.gz -o ./restore
  arx -p ./src backup.zip
  arx -p ./src ./docs backup.tar.gz
  arx -z ./src backup.zip
  arx -t ./src backup.tar
  arx -T ./src backup.tar.gz
  arx -l archive.tar
  arx -i archive.zip
  arx -r archive.zip old.txt new.txt",
    after_help = "Проект: arx · лицензия MIT"
)]
pub struct Cli {
    /// Распаковать архив (формат — авто)
    #[arg(short = 'u', long = "unpack", group = "action")]
    pub unpack: bool,

    /// Упаковать: формат выбирается по расширению выходного файла
    #[arg(short = 'p', long = "pack", group = "action")]
    pub pack: bool,

    /// Упаковать в zip
    #[arg(short = 'z', long = "zip", group = "action")]
    pub zip: bool,

    /// Упаковать в tar
    #[arg(short = 't', long = "tar", group = "action")]
    pub tar: bool,

    /// Упаковать в tar.gz
    #[arg(short = 'T', long = "targz", group = "action")]
    pub tar_gz: bool,

    /// Показать список файлов в архиве
    #[arg(short = 'l', long = "list", group = "action")]
    pub list: bool,

    /// Показать информацию об архиве
    #[arg(short = 'i', long = "info", group = "action")]
    pub info: bool,

    /// Переименовать файл внутри архива: -r <archive> <from> <to>
    #[arg(short = 'r', long = "rename", group = "action", num_args = 3)]
    pub rename: Option<Vec<String>>,

    /// Уровень сжатия 0..9 (0 = без сжатия, 9 = максимум). По умолчанию 6.
    #[arg(short = 'L', long = "level", value_parser = clap::value_parser!(u32).range(0..=9), default_value_t = 6)]
    pub level: u32,

    /// Исключить путь или маску (можно указывать несколько раз)
    #[arg(long = "exclude", value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// Показывать список упакованных файлов
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Тихий режим (без прогресс-бара)
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Источники (для упаковки) или архив (для -u/-l/-i).
    /// Последний аргумент — имя архива, если он с расширением .zip/.tar/.tar.gz/.tgz
    #[arg(value_name = "INPUT", num_args = 1..)]
    pub input: Vec<PathBuf>,

    /// Куда распаковывать (только для -u), по умолчанию ./extracted
    #[arg(short = 'o', long = "out", default_value = "./extracted", value_name = "DIR")]
    pub out_dir: PathBuf,
}
