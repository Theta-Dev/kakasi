use crate::{Records, CLETTERS};

const ENDMARK: [char; 11] = [
    ')', ']', '!', '.', ',', '\u{3001}', '\u{3002}', '\u{ff1f}', '\u{ff10}', '\u{ff1e}', '\u{ff1c}',
];
const DASH_SYMBOLS: [char; 4] = ['\u{30FC}', '\u{2015}', '\u{2212}', '\u{FF70}'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharType {
    Kanji,
    Katakana,
    Hiragana,
    Symbol,
    Alpha,
}

pub fn convert(text: &str, dict: &Records) -> String {
    // TODO: char conversion should be done with iterators
    let mut char_indices = text.char_indices();
    let mut kana_text = String::new();
    let mut hiragana = String::new();
    let mut prev_type = CharType::Kanji;

    // output_flag
    // means (output buffer?, output text[i]?, copy to buffer and increment i?)
    // possible (False, True, True), (True, False, False), (True, True, True)
    //          (False, False, True)

    while let Some((i, c)) = char_indices.next() {
        let output_flag = if ENDMARK.contains(&c) {
            (CharType::Symbol, true, true, true)
        } else if DASH_SYMBOLS.contains(&c) {
            (prev_type, false, false, true)
        } else if is_sym(c) {
            if prev_type != CharType::Symbol {
                (CharType::Symbol, true, false, true)
            } else {
                (CharType::Symbol, false, true, true)
            }
        } else if wana_kana::utils::is_char_katakana(c) {
            (
                CharType::Katakana,
                prev_type != CharType::Katakana,
                false,
                true,
            )
        } else if wana_kana::utils::is_char_hiragana(c) {
            (
                CharType::Hiragana,
                prev_type != CharType::Hiragana,
                false,
                true,
            )
        } else if c.is_ascii() {
            (CharType::Alpha, prev_type != CharType::Alpha, false, true)
        } else if wana_kana::utils::is_char_kanji(c) {
            if !kana_text.is_empty() {
                hiragana.push_str(&convert_kana(&kana_text));
            }
            let (t, n) = convert_kanji(&text[i..], &kana_text, &dict);

            if n > 0 {
                kana_text = t;
                for _ in 1..n {
                    char_indices.next();
                }
                (CharType::Kanji, false, false, false)
            } else {
                // Unknown kanji
                kana_text.clear();
                // TODO: FOR TESTING
                hiragana.push_str("🯄");
                (CharType::Kanji, true, false, false)
            }
        } else if matches!(c as u32, 0xf000..=0xfffd | 0x10000..=0x10ffd) {
            // PUA: ignore and drop
            if !kana_text.is_empty() {
                hiragana.push_str(&convert_kana(&kana_text));
            }
            (prev_type, false, false, false)
        } else {
            (prev_type, true, true, true)
        };

        prev_type = output_flag.0;

        if output_flag.1 && output_flag.2 {
            kana_text.push(c);
            hiragana.push_str(&convert_kana(&kana_text));
            kana_text.clear()
        } else if output_flag.1 && output_flag.3 {
            if !kana_text.is_empty() {
                hiragana.push_str(&convert_kana(&kana_text));
            }
            kana_text = c.to_string();
        } else if output_flag.3 {
            kana_text.push(c);
        }
    }

    // Convert last word
    if !kana_text.is_empty() {
        hiragana.push_str(&convert_kana(&kana_text));
    }

    hiragana
}

fn is_sym(c: char) -> bool {
    matches!(c as u32,
        0x3000..=0x3020 |
        0x3030..=0x303F |
        0x0391..=0x03A1 |
        0x03A3..=0x03A9 |
        0x03B1..=0x03C9 |
        0x0410..= 0x044F |
        0xFF01..=0xFF1A |
        0x00A1..=0x00FF |
        0xFF20..=0xFF5E |
        0x0451 |
        0x0401
    )
}

fn convert_kana(text: &str) -> String {
    wana_kana::to_hiragana::to_hiragana_with_opt(
        text,
        wana_kana::Options {
            use_obsolete_kana: false,
            pass_romaji: true,
            upcase_katakana: false,
            imemode: false,
        },
    )
}

/// Convert the leading kanji from the input string to hiragana
fn convert_kanji(text: &str, btext: &str, dict: &Records) -> (String, usize) {
    let mut translation: Option<String> = None;
    let mut i_c = 0;
    let mut n_c = 0;
    let mut char_indices = text.char_indices().peekable();

    while let Some((i, c)) = char_indices.next() {
        let kanji = &text[0..i + c.len_utf8()];

        let this_tl = match dict.get(kanji) {
            Some(readings) => {
                readings
                .iter()
                .find_map(|(k, reading)| {
                    if k.is_empty() {
                        None
                    } else if let Some(cltr) = CLETTERS.get(&k.chars().next().unwrap_or_default()) {
                        char_indices.peek().and_then(|(_, next_c)| {
                            // Shortcut if the next character is not hiragana
                            if wana_kana::utils::is_char_hiragana(*next_c) {
                                if cltr.contains(&&next_c.to_string().as_str()) {
                                    // Add the next character to the char count
                                    i_c += 1;
                                    let mut hira = reading.to_owned();
                                    hira.push(*next_c);
                                    return Some(hira);
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    } else if wana_kana::is_hiragana::is_hiragana(&k) {
                        if btext.contains(reading) {
                            Some(reading.to_owned())
                        } else {
                            None
                        }
                    } else {
                        panic!("invalid reading key")
                    }
                })
                .or_else(|| readings.get("").cloned())
            },
            None => break,
        };

        i_c += 1;
        if let Some(tl) = this_tl {
            translation = Some(tl);
            n_c = i_c;
        }
    }

    translation
        .map(|tl| (tl.to_owned(), n_c))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("会っAbc", "あっ", 2)]
    #[case("渋谷", "しぶや", 2)]
    #[case("東北大学電気通信研究所", "とうほくだいがくでんきつうしんけんきゅうじょ", 11)]
    #[case("暑中お見舞い申し上げます", "しょちゅうおみまいもうしあげます", 12)]
    fn t_convert_kanji(#[case] text: &str, #[case] expect: &str, #[case] expect_n: usize) {
        let dict = crate::get_kanji_dict();
        let (res, n) = convert_kanji(text, "", &dict);
        assert_eq!(res, expect);
        assert_eq!(n, expect_n);
    }
}
