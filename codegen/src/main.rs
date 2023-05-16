mod phfbin_gen;
mod testconv;

use std::{borrow::Cow, collections::BTreeMap, path::Path};

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
    if line.is_empty() || line.starts_with(";;") {
        return;
    }

    let normalized = normalize(line);
    let mut token = normalized.split_ascii_whitespace();
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
            // Replace iteration characters (`々`)
            let kanji = kanji
                .char_indices()
                .map(|(i, c)| {
                    if c == '々' {
                        if let Some(prev_c) = kanji[0..i].chars().last() {
                            prev_c
                        } else {
                            panic!(
                                "kanhi({}): could not replace iteration char, `{}`",
                                ln, line
                            )
                        }
                    } else {
                        c
                    }
                })
                .collect::<String>();

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

            let record = records.entry(kanji).or_default();
            record
                .entry(
                    tail.map(|t| t.to_string())
                        .or_else(|| context.map(str::to_owned))
                        .unwrap_or_default(),
                )
                .or_insert_with(|| reading.to_owned());
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

type Records = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Default)]
struct KanjiString(String);

#[derive(Default)]
struct Readings(BTreeMap<String, String>);

impl Encodable for KanjiString {
    fn encode(&self) -> Vec<u8> {
        self.0
            .chars()
            .flat_map(|c| {
                let c = c as u32;
                if c > 0xffff {
                    panic!("character `{}` > 0xffff", { c });
                }
                [((c & 0xff00) >> 8) as u8, (c & 0xff) as u8]
            })
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

fn insert_placeholders(records: &mut Records) {
    let mut to_insert = Vec::new();

    for kanji in records.keys() {
        for (i, c) in kanji.char_indices() {
            let sub = &kanji[0..i + c.len_utf8()];
            if !records.contains_key(sub) {
                to_insert.push(sub.to_owned());
            }
        }
    }

    println!("kanji_dict: {} placeholders", to_insert.len());
    to_insert.into_iter().for_each(|s| {
        records.entry(s).or_default();
    });
}

fn get_kanji_dict() -> Records {
    let mut records = Records::default();
    parse_dict(&mut records, Path::new("dict/kakasidict.utf8"));
    records = find_redundant_compounds(&records);
    println!("kanji_dict: {} entries", records.len());
    insert_placeholders(&mut records);
    println!("kanji_dict: {} total items", records.len());
    records
}

fn generate_kanji_dict() -> Vec<u8> {
    let records = get_kanji_dict();

    let mut phfmap = phfbin_gen::Map::<KanjiString, Readings>::default();
    for (kanji, readings) in records {
        phfmap.entry(KanjiString(kanji), Readings(readings));
    }
    phfmap.build()
}

fn export_kanji_dict() -> String {
    let records = get_kanji_dict();
    let mut buf = String::new();

    for (kanji, readings) in records {
        for (key, hira) in readings {
            if key.is_ascii() {
                assert!(key.len() <= 1);
                buf.push_str(&hira);
                buf.push_str(&key);
                buf.push(' ');
                buf.push_str(&kanji);
            } else {
                buf.push_str(&hira);
                buf.push(' ');
                buf.push_str(&kanji);
                buf.push(' ');
                buf.push_str(&key);
            }
            buf.push('\n');
        }
    }
    buf
}

fn generate_syn_dict() -> String {
    let mut dict = BTreeMap::new();
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

fn generate_rom_dict() -> String {
    let mut dict = BTreeMap::new();
    let mut max_klen = 0;
    let content = std::fs::read_to_string("dict/hepburn.utf8").unwrap();
    content
        .lines()
        .for_each(|line| parse_rom_ln(&mut dict, &mut max_klen, line));

    let mut phf_map = phf_codegen::Map::<&str>::new();

    for (key, val) in &dict {
        phf_map.entry(key, &format!("{:?}", val));
    }

    format!(
        "pub const HEPBURN_MAX_KLEN: usize = {};\n\n#[rustfmt::skip]\npub static HEPBURN_DICT: phf::Map<&str, &str> = {};\n",
        max_klen,
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

fn parse_syn_ln(dict: &mut BTreeMap<char, char>, line: &str) {
    // Skip comments
    if line.is_empty() || line.starts_with(";;") {
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

fn parse_rom_ln(dict: &mut BTreeMap<String, String>, max_klen: &mut usize, line: &str) {
    // Skip comments
    if line.is_empty() || line.starts_with(";;") {
        return;
    }

    let mut token = line.split_ascii_whitespace();
    let value = token.next();
    let key = token.next();

    match (key, value) {
        (Some(key), Some(value)) => {
            *max_klen = key.chars().count().max(*max_klen);

            dict.entry(key.to_owned())
                .or_insert_with(|| value.to_owned());
        }
        _ => panic!("rom: could not parse line: `{}`", line),
    }
}

const ITERATION_MARK: char = '々';

/// NFKC-normalize and replace iteration marks
fn normalize(text: &str) -> String {
    let mut imcount = 0;
    let replacements = text.char_indices().filter_map(|(i, c)| {
        if c == ITERATION_MARK {
            // Count iteration marks
            if imcount == 0 {
                imcount = 1;
                for c in text[i + c.len_utf8()..].chars() {
                    if c == ITERATION_MARK {
                        imcount += 1;
                    } else {
                        break;
                    }
                }
            }

            // Replace withe the character imcount positions before
            text[0..i]
                .chars()
                .rev()
                .nth(imcount - 1)
                .map(|prev| (i, c.len_utf8(), prev))
        } else {
            imcount = 0;
            None
        }
    });

    let mut new = String::with_capacity(text.len());
    let mut last = 0;

    for (i, clen, r_char) in replacements {
        new.extend(text[last..i].nfkc());
        new.push(r_char);
        last = i + clen;
    }
    new.extend(text[last..].nfkc());
    new
}

fn main() {
    let mut args = std::env::args();
    args.next();
    let arg = args.next().unwrap_or_default();

    if arg == "export" {
        let kd = export_kanji_dict();
        std::fs::write("kd.utf8", kd).unwrap();
    } else {
        let code_header = r#"// This file is automatically generated using the kakasi-codegen crate. DO NOT EDIT.
"#;
        let kanji_dict_bytes = generate_kanji_dict();
        std::fs::write("kanji_dict.bin", kanji_dict_bytes).unwrap();

        let code_syn_dict = generate_syn_dict();
        let code = format!("{}\n{}", code_header, code_syn_dict);
        std::fs::write("syn_dict.rs", code).unwrap();

        let code_rom_dict = generate_rom_dict();
        let code = format!("{}\n{}", code_header, code_rom_dict);
        std::fs::write("hepburn_dict.rs", code).unwrap();
    }
}
