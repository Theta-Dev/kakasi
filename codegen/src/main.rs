mod phfbin_gen;
mod testconv;

use std::{borrow::Cow, collections::HashMap, path::Path};

use once_cell::sync::Lazy;
use phf::PhfHash;
use phfbin_gen::Encodable;
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
        panic!("kanji({}): more than 1 ctx, `{}`", ln, line);
    }

    if let Some(context) = context {
        if !wana_kana::is_hiragana::is_hiragana(context) {
            panic!("kanji({}): ctx not hiragana, `{}`", ln, line);
        }
    }

    match (reading, kanji) {
        (Some(mut reading), Some(kanji)) => {
            // Parse tail
            let (i_last, last) = reading.char_indices().last().unwrap();
            let tail = if last.is_ascii_alphabetic() {
                reading = &reading[0..i_last];
                if !CLETTERS.contains_key(&last) {
                    panic!("kanji({}): invalid tail, `{}`", ln, line);
                }
                Some(last)
            } else {
                None
            };

            if !wana_kana::is_hiragana::is_hiragana(reading) {
                panic!("kanji({}): reading not hiragana", ln);
            }

            if tail.is_some() && context.is_some() {
                panic!("kanji({}): tail + context are mutually exclusive", ln);
            }

            let record = records.entry(kanji.to_owned()).or_default();
            match record.entry(
                tail.map(|t| t.to_string())
                    .or_else(|| context.map(str::to_owned))
                    .unwrap_or_default(),
            ) {
                std::collections::hash_map::Entry::Occupied(_) => {
                    /*
                    // Replace reading if the new one is shorter
                    let val = e.get_mut();
                    if val.len() > reading.len() {
                        *val = reading.to_owned();
                    }*/
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(reading.to_owned());
                }
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

type Records = HashMap<String, HashMap<String, String>>;

#[derive(Default)]
struct KanjiString(String);

#[derive(Default)]
struct Readings(HashMap<String, String>);

impl Encodable for KanjiString {
    fn encode(&self) -> Vec<u8> {
        self.0
            .encode_utf16()
            .flat_map(|c16| [((c16 & 0xff00) >> 8) as u8, (c16 & 0xff) as u8])
            .collect()
    }
}

impl PhfHash for KanjiString {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.phf_hash(state)
    }
}

fn enc_hiragana(buf: &mut Vec<u8>, text: &str) {
    text.chars().for_each(|c| {
        if wana_kana::utils::is_char_hiragana(c) {
            let c_enc = match c {
                '\u{30fc}' => 0x7f,
                _ => {
                    let c_enc = (c as u32 - wana_kana::constants::HIRAGANA_START)
                        .try_into()
                        .unwrap();
                    if c_enc > 127 {
                        panic!("char `{}` > 127", c);
                    }
                    c_enc
                }
            };
            buf.push(c_enc);
        } else {
            panic!("text `{}` is not pure hiragana", text)
        }
    });
}

impl Encodable for Readings {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        for (k, reading) in &self.0 {
            // The default reading is encoded at the last position
            if k.is_empty() {
                continue;
            }

            // Add separator
            if !buf.is_empty() {
                buf.push(0xff);
            }

            if k.is_ascii() {
                // Tail: `kか`
                if k.len() != 1 {
                    panic!("invalid tail: `{}`", k);
                }

                buf.push(k.as_bytes()[0] | 0x80);
                enc_hiragana(&mut buf, reading);
            } else {
                // Context: `しょくSそん`
                enc_hiragana(&mut buf, reading);
                buf.push(0x80);
                enc_hiragana(&mut buf, k);
            }
        }

        // Default reading last
        if let Some(reading) = self.0.get("") {
            // Add separator
            if !buf.is_empty() {
                buf.push(0xff);
            }

            enc_hiragana(&mut buf, reading);
        }

        buf
    }
}

fn find_redundant_compounds(dict: &Records) -> Records {
    let mut wdict = dict.clone();

    for (kanji, readings) in dict {
        if kanji.chars().count() <= 3 {
            continue;
        }
        if readings.len() != 1 {
            continue;
        }

        if let Some(reading) = readings.get("") {
            // Try to convert the entry without it being present
            let entry = wdict.remove_entry(kanji).unwrap();
            let res = testconv::convert(kanji, &wdict);

            if &res == reading || to_romaji_nodc(&res) == to_romaji_nodc(reading) {
                println!("Redundant: {} - {}", kanji, reading);
            } else {
                // Put the entry back if it is necessary
                wdict.insert(entry.0, entry.1);
            }
        }
    }
    wdict
}

/// Romanize and remove double consonants
fn to_romaji_nodc(text: &str) -> String {
    let rom = wana_kana::to_romaji::to_romaji(text);

    let mut buf = String::new();
    let mut citer = rom.chars().peekable();

    while let Some(c) = citer.next() {
        if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
            match citer.peek() {
                Some(nc) => {
                    if &c != nc {
                        buf.push(c);
                    }
                }
                None => buf.push(c),
            }
        } else {
            buf.push(c);
        }
    }
    buf
}

fn generate_kanji_dict() -> Vec<u8> {
    let mut records = Records::default();
    parse_dict(&mut records, Path::new("dict/kakasidict.utf8"));
    records = find_redundant_compounds(&records);

    println!("kanji_dict: {} entries", records.len());

    let mut phfmap = phfbin_gen::Map::<KanjiString, Readings>::default();
    for (kanji, readings) in records {
        phfmap.entry(KanjiString(kanji), Readings(readings));
    }
    phfmap.build()
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
        "#[rustfmt::skip]\npub(crate) static SYN_DICT: phf::Map<char, char> = {};\n",
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
    let kanji_dict_bytes = generate_kanji_dict();
    std::fs::write("kanji_dict.bin", &kanji_dict_bytes).unwrap();

    let code_syn_dict = generate_syn_dict();
    let code = format!("{}\n{}", code_header, code_syn_dict);
    std::fs::write("syn_dict.rs", &code).unwrap();
}
