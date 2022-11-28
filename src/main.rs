use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

fn convert_line(line: &str, hira: bool) {
    let res = kakasi::convert(line);
    if hira {
        println!("{}", res.hiragana);
    } else {
        println!("{}", res.romaji);
    }
}

fn main() {
    // Parse commandline arguments
    let mut hira = false;
    let mut path = None;
    let mut txt = String::new();

    let mut args = std::env::args();
    let mut after_opts = false;
    let pname = args.next().unwrap_or_else(|| "kakasi".to_owned());
    while let Some(a) = args.next() {
        if !after_opts && a.starts_with('-') {
            if a == "-h" || a == "--help" {
                println!(
                    r#"Transliterate hiragana, katakana and kanji (Japanese text) into romaji (Latin alphabet).

Usage: {} [OPTION] [Japanese text]

With no file or text given, kakasi reads from STDIN.

Options:
  -f <FILE> Read from text file
  -k  Transliterate to hiragana instead of romaji
  -h  show this help page"#,
                    pname
                );
                return;
            } else if a == "-k" {
                hira = true;
            } else if a == "-f" {
                match args.next() {
                    Some(p) => path = Some(PathBuf::from(p)),
                    None => {
                        eprintln!("no file path given");
                        std::process::exit(2);
                    }
                }
            } else {
                continue;
            }
        } else {
            after_opts = true;
            if !txt.is_empty() {
                txt.push(' ');
            }
            txt += &a;
        }
    }

    match path {
        Some(path) => {
            let f = match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("could not open {}, error: {}", path.to_string_lossy(), e);
                    std::process::exit(1);
                }
            };
            BufReader::new(f)
                .lines()
                .flatten()
                .for_each(|l| convert_line(&l, hira));
        }
        None => {
            if txt.is_empty() {
                std::io::stdin()
                    .lines()
                    .flatten()
                    .for_each(|l| convert_line(&l, hira))
            } else {
                txt.lines().for_each(|l| convert_line(l, hira));
            }
        }
    };
}
