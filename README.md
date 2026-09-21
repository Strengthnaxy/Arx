# arx

Один инструмент — zip, tar, tar.gz. Упаковка, распаковка, просмотр.

## Возможности

- **упаковать** папки и файлы в zip / tar / tar.gz
- **распаковать** любые из этих форматов (авто-определение)
- **список** файлов внутри архива
- **информация** об архиве (формат, размер, кол-во)
- **переименование** файла внутри архива
- уровень сжатия `-L 0..9`
- исключения `--exclude`
- несколько источников в одном архиве

## Флаги

| Флаг | Значение |
|------|----------|
| `-u` | unpack — распаковать |
| `-p` | pack — упаковать, формат по имени выходного файла |
| `-z` | упаковать в zip |
| `-t` | упаковать в tar |
| `-T` | упаковать в tar.gz |
| `-l` | list — список файлов |
| `-i` | info — информация |
| `-r` | rename — переименовать внутри |
| `-L N` | уровень сжатия 0..9 (по умолчанию 6) |
| `--exclude P` | исключить файлы (можно несколько раз) |
| `-o DIR` | куда распаковать (для `-u`) |
| `-v` | показать, что упаковывается |
| `-q` | тихий режим |
| `-h` | help |
| `-V` | version |

## Примеры

    # распаковка (формат авто)
    arx -u backup.zip
    arx -u backup.tar.gz -o ./restore

    # упаковка: авто-формат по имени архива
    arx -p ./src backup.zip
    arx -p ./src backup.tar
    arx -p ./src backup.tar.gz

    # упаковка нескольких источников в один архив
    arx -p ./src ./docs ./README.md project.tar.gz

    # принудительный формат
    arx -z ./src backup.zip
    arx -t ./src backup.tar
    arx -T ./src backup.tar.gz

    # уровень сжатия
    arx -p ./src backup.tar.gz -L 9
    arx -p ./src backup.zip -L 0

    # исключения
    arx -p ./project backup.zip --exclude target --exclude "*.log"

    # список, инфа, переименование
    arx -l backup.zip
    arx -i backup.zip
    arx -r backup.zip old.txt new.txt

## Сборка

    cargo build --release

## Кроссплатформенность

- Linux / macOS: `target/release/arx`
- Windows: `cargo build --release --target x86_64-pc-windows-gnu` → `arx.exe`
