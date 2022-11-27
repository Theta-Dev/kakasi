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

    let mut args = std::env::args();
    let pname = args.next().unwrap_or_else(|| "kakasi".to_owned());
    for a in args {
        if a.starts_with('-') {
            if a == "-h" || a == "--help" {
                println!(
                    r#"Transliterate hiragana, katakana and kanji (Japanese text) into romaji (Latin alphabet).

Usage: {} [OPTION] [FILE]

With no FILE, read standard input.

Options:
  -k  Transliterate to hiragana instead of romaji
  -h  show this help page"#,
                    pname
                );
                return;
            } else if a == "-k" {
                hira = true;
            } else {
                continue;
            }
        } else if path.is_none() {
            path = Some(PathBuf::from(a));
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
        None => std::io::stdin()
            .lines()
            .flatten()
            .for_each(|l| convert_line(&l, hira)),
    };
}
