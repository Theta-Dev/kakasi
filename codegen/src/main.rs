use std::{borrow::Cow, collections::HashMap, path::Path};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use unicode_normalization::UnicodeNormalization;

fn parse_dict<P: AsRef<Path>>(records: &mut Records, file: P) {
    let content = std::fs::read_to_string(file).unwrap().nfkc().to_string();
    content
        .lines()
        .enumerate()
        .for_each(|(ln, line)| parse_dict_ln(records, line, ln + 1));
}

fn parse_dict_ln(records: &mut Records, line: &str, ln: usize) {
    // Skip comments
    if line.starts_with(";;") || line.is_empty() {
        return;
    }

    let mut token = line.split_ascii_whitespace();
    let reading = token.next();
    let kanji = token.next();
    let context = token.next();

    // Validate
    if token.next().is_some() {
        println!("kanji({}): more than 1 ctx, `{}`", ln, line);
    }

    if let Some(context) = context {
        if !wana_kana::is_hiragana::is_hiragana(context) {
            println!("kanji({}): ctx not hiragana, `{}`", ln, line);
        }
    }

    match (reading, kanji) {
        (Some(mut reading), Some(kanji)) => {
            // Parse tail
            let (i_last, last) = reading.char_indices().last().unwrap();
            let tail = if last.is_ascii_alphabetic() {
                reading = &reading[0..i_last];
                if !CLETTERS.contains_key(&last) {
                    println!("kanji({}): invalid tail, `{}`", ln, line);
                }
                Some(last)
            } else {
                None
            };

            if !wana_kana::is_hiragana::is_hiragana(reading) {
                println!("kanji({}): reading not hiragana", ln);
            }
            if !wana_kana::is_kanji::contains_kanji(kanji)
                || kanji.chars().any(|c| c.is_ascii() && c != ' ')
            {
                println!("kanji({}): kanji not kanji/hiragana, `{}`", ln, line);
            }
        }
        _ => panic!("kanji({}): could not parse line, `{}`", ln, line),
    }
}

static CLETTERS: phf::Map<char, &[&str]> = phf::phf_map!(
    'a' => &["あ", "ぁ", "っ", "わ", "ゎ"],
    'i' => &["い", "ぃ", "っ", "ゐ"],
    'u' => &["う", "ぅ", "っ"],
    'e' => &["え", "ぇ", "っ", "ゑ"],
    'o' => &["お", "ぉ", "っ"],
    'k' => &["か", "ゕ", "き", "く", "け", "ゖ", "こ", "っ"],
    'g' => &["が", "ぎ", "ぐ", "げ", "ご", "っ"],
    's' => &["さ", "し", "す", "せ", "そ", "っ"],
    'z' => &["ざ", "じ", "ず", "ぜ", "ぞ", "っ"],
    'j' => &["ざ", "じ", "ず", "ぜ", "ぞ", "っ"],
    't' => &["た", "ち", "つ", "て", "と", "っ"],
    'd' => &["だ", "ぢ", "づ", "で", "ど", "っ"],
    'c' => &["ち", "っ"],
    'n' => &["な", "に", "ぬ", "ね", "の", "ん"],
    'h' => &["は", "ひ", "ふ", "へ", "ほ", "っ"],
    'b' => &["ば", "び", "ぶ", "べ", "ぼ", "っ"],
    'f' => &["ふ", "っ"],
    'p' => &["ぱ", "ぴ", "ぷ", "ぺ", "ぽ", "っ"],
    'm' => &["ま", "み", "む", "め", "も"],
    'y' => &["や", "ゃ", "ゆ", "ゅ", "よ", "ょ"],
    'r' => &["ら", "り", "る", "れ", "ろ"],
    'w' => &["わ", "ゐ", "ゑ", "ゎ", "を", "っ"],
    'v' => &["ゔ"],
);

type Records = HashMap<String, String>;

fn updaterec(
    records: &mut Records,
    kanji: &str,
    reading: &str,
    tail: Option<char>,
    context: &[&str],
) {
    if !context.is_empty() {
        eprintln!("skipping `{}` with context {:?}", kanji, context);
        return;
    }

    match tail {
        Some(tail) => {
            if let Some(cltrs) = CLETTERS.get(&tail) {
                cltrs.iter().for_each(|c| {
                    updaterec(
                        records,
                        &format!("{}{}", kanji, c),
                        &format!("{}{}", reading, c),
                        None,
                        context,
                    )
                });
            } else {
                panic!("invalid tail: {}", tail);
            }
        }
        None => {
            records
                .entry(kanji.to_owned())
                .or_insert_with(|| reading.to_owned());
        }
    }
}

fn generate_kanji_dict() -> String {
    let mut records = Records::default();
    parse_dict(&mut records, Path::new("dict/kakasidict.utf8"));

    String::new()
}

fn generate_syn_dict() -> String {
    let mut dict = HashMap::new();
    let content = std::fs::read_to_string("dict/itaijidict.utf8").unwrap();
    content
        .lines()
        .for_each(|line| parse_syn_ln(&mut dict, line));

    let mut phf_map = phf_codegen::Map::<char>::new();

    for (key, val) in &dict {
        phf_map.entry(*key, &format!("{:?}", val));
    }

    format!(
        "#[rustfmt::skip]\npub static SYN_DICT: phf::Map<char, char> = {};\n",
        phf_map.build()
    )
}

fn unescape(text: &str) -> Cow<str> {
    static ESCAPE_SEQUENCE_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\\[Uu]([\dA-Fa-f]{4,8})"#).unwrap());

    ESCAPE_SEQUENCE_RE.replace_all(text, |caps: &Captures| {
        let hex_str = caps.get(1).unwrap().as_str();
        match u32::from_str_radix(hex_str, 16) {
            Ok(hex_val) => match char::from_u32(hex_val) {
                Some(c) => c.to_string(),
                None => panic!("could not convert character {}", hex_str),
            },
            Err(_) => panic!("could not parse character {}", hex_str),
        }
    })
}

fn parse_syn_ln(dict: &mut HashMap<char, char>, line: &str) {
    // Skip comments
    if line.starts_with(";;") || line.is_empty() {
        return;
    }

    let line_unescaped = unescape(line);
    let mut token = line_unescaped.split_ascii_whitespace();
    let value = token.next();
    let key = token.next();

    match (key, value) {
        (Some(key), Some(value)) => {
            if key.is_empty() || value.is_empty() {
                panic!("invalid line: `{}`", line);
            }

            let mut kchars = key.nfkc();
            let mut vchars = value.nfkc();

            let kc = kchars.next().unwrap();
            let vc = vchars.next().unwrap();

            if kc == vc {
                eprintln!("syn: equal k/v `{}`, skipping", kc);
                return;
            }

            if kchars.next().is_some() || vchars.next().is_some() {
                panic!("syn: invalid line, k/v has more than 1 char: `{}`", line);
            }

            dict.insert(kc, vc);
        }
        _ => panic!("syn: could not parse line: `{}`", line),
    }
}

fn main() {
    let code_header = r#"// This file is automatically generated using the kakasi-codegen crate. DO NOT EDIT.
"#;
    let code_kanji_dict = generate_kanji_dict();
    let code_syn_dict = generate_syn_dict();

    let code = format!("{}\n{}\n{}", code_header, code_kanji_dict, code_syn_dict);

    std::fs::write("dict.rs", &code).unwrap();
}
