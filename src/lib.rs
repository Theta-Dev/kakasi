#![doc = include_str!("../README.md")]
#![warn(missing_docs, clippy::todo)]

mod hepburn_dict;
mod phfbin;
mod syn_dict;
mod types;
mod util;

pub use types::{IsJapanese, KakasiResult};

use unicode_normalization::UnicodeNormalization;

use phfbin::PhfMap;
use types::{CharType, KanjiString, Readings};

/// Convert the given Japanese text to hiragana/romaji
///
/// ```
/// let res = kakasi::convert("Hello 日本!");
/// assert_eq!(res.hiragana, "Hello にほん!");
/// assert_eq!(res.romaji, "Hello nihon !");
/// ```
pub fn convert(text: &str) -> KakasiResult {
    let dict = PhfMap::new(util::KANJI_DICT);

    let text = normalize(text);

    let mut char_indices = text.char_indices().peekable();
    let mut kana_buf = String::new();
    // Type of the character last added to kana_buf
    let mut prev_buf_type = CharType::Whitespace;
    // Type of the character last added to the result
    let mut prev_acc_type = CharType::Whitespace;
    // Capitalization flags
    // 0: capitalize next word, 1: capitalize first sentence, 2: first sentence capitalized
    let mut cap = (false, false, false);

    let mut res = KakasiResult::new(text.len());

    let conv_kana_buf = |kana_buf: &mut String,
                         res: &mut KakasiResult,
                         prev_acc_type: &mut CharType,
                         cap: &mut (bool, bool, bool)| {
        if !kana_buf.is_empty() {
            let hira = convert_katakana(kana_buf);
            res.hiragana.push_str(&hira);
            let mut rom = hiragana_to_romaji(&hira);

            if cap.0 {
                rom = util::capitalize_first_c(&rom);
                cap.0 = false;
            }
            if cap.1 && !cap.2 {
                res.romaji = util::capitalize_first_c(&res.romaji);
                cap.2 = true;
            }

            util::ensure_trailing_space(
                &mut res.romaji,
                *prev_acc_type != CharType::LeadingPunct
                    && *prev_acc_type != CharType::JoiningPunct,
            );
            res.romaji.push_str(&rom);

            kana_buf.clear();
            *prev_acc_type = CharType::Hiragana;
        }
    };

    while let Some((i, c)) = char_indices.next() {
        if util::is_char_in_range(c, util::HIRAGANA) {
            if prev_buf_type != CharType::Hiragana {
                conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
            }
            kana_buf.push(c);
            prev_buf_type = CharType::Hiragana;
        } else if util::is_char_in_range(c, util::KATAKANA) {
            if prev_buf_type != CharType::Katakana {
                conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
            }
            kana_buf.push(c);
            prev_buf_type = CharType::Katakana;
        } else if util::is_char_in_range(c, util::KANJI) {
            let (t, n) = convert_kanji(&text[i..], &kana_buf, &dict);
            conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);

            if n > 0 {
                kana_buf = t;
                conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
                for _ in 1..n {
                    char_indices.next();
                }
            } else {
                // Unknown kanji
                res.hiragana.push(c);
                res.romaji.push(c);
            }
            prev_acc_type = CharType::Kanji;
        } else if c.is_whitespace() {
            conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
            res.hiragana.push(c);
            res.romaji.push(c);
            prev_acc_type = CharType::Whitespace;
        } else if c == '・' {
            conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
            res.hiragana.push(c);
            res.romaji.push(' ');
            prev_acc_type = CharType::Whitespace;
        } else if c == util::PROLONGED_SOUND_MARK {
            if prev_buf_type != CharType::Hiragana && prev_buf_type != CharType::Katakana {
                conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
            }
            kana_buf.push(c);
            prev_buf_type = match prev_buf_type {
                CharType::Hiragana => CharType::Hiragana,
                _ => CharType::Katakana,
            };
        } else {
            // The rest. Latin characters, other scripts, numbers, special characters
            conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
            res.hiragana.push(c);

            // Determine the character type (required for correct spacing and capitalization).
            // Japanese punctuation can be looked up in the dictionary, otherwise assume CharType::Other.
            // Special case: dots and commas used as decimal seperators
            let (c_rom, char_type) = util::PCT_DICT.get(&c).copied().unwrap_or_else(|| {
                (
                    c,
                    if c.is_ascii_digit()
                        || ((c == '.' || c == ',')
                            && prev_acc_type == CharType::Numeric
                            && char_indices
                                .peek()
                                .map(|(_, nc)| nc.is_ascii_digit())
                                .unwrap_or_default())
                    {
                        CharType::Numeric
                    } else {
                        CharType::Other
                    },
                )
            });

            let is_jpunct = util::is_char_japanese_punctuation(c);
            if (prev_acc_type != CharType::Other
                && prev_acc_type != CharType::Numeric
                && prev_acc_type != CharType::Whitespace)
                || is_jpunct
            {
                util::ensure_trailing_space(
                    &mut res.romaji,
                    prev_acc_type != CharType::LeadingPunct
                        && prev_acc_type != CharType::JoiningPunct
                        && char_type != CharType::TrailingPunct
                        && char_type != CharType::JoiningPunct,
                );
            }

            // Japanese punctuation was not normalized at the beginning,
            // the normalization here will replace fullwidth characters with normal ones.
            if is_jpunct && char_type == CharType::Other {
                res.romaji.extend(c_rom.nfkc());
            } else {
                res.romaji.push(c_rom);
            }

            // If the current character is a full stop (no decimal point),
            // the next word should be capitalized.
            // Keep the capitalization flag set if the following character is leading or joining
            // punctuation. Example: `Sentence1. "Nice", sentence 2.`
            cap.0 = c_rom == '.' && char_type != CharType::Numeric
                || cap.0 && matches!(char_type, CharType::LeadingPunct | CharType::JoiningPunct);
            cap.1 |= cap.0;

            prev_acc_type = char_type;
        };
    }

    conv_kana_buf(&mut kana_buf, &mut res, &mut prev_acc_type, &mut cap);
    res
}

/// Check if the input text is Japanese
///
/// Note that (especially very short) japanese texts are not always
/// distinguishable from Chinese, because these languages use the same
/// characters.
///
/// Thus if only CJK ideographs are found, the function returns
/// [`IsJapanese::Maybe`].
///
/// ```
/// # use kakasi::IsJapanese;
/// assert_eq!(kakasi::is_japanese("Abc"), IsJapanese::False);
/// assert_eq!(kakasi::is_japanese("日本"), IsJapanese::Maybe);
/// assert_eq!(kakasi::is_japanese("ラスト"), IsJapanese::True);
/// ```
pub fn is_japanese(text: &str) -> IsJapanese {
    let mut maybe = false;
    for c in text.chars() {
        if util::is_char_in_range(c, util::HIRAGANA) || util::is_char_in_range(c, util::KATAKANA) {
            return IsJapanese::True;
        }
        maybe |= util::is_char_in_range(c, util::KANJI);
    }
    match maybe {
        true => IsJapanese::Maybe,
        false => IsJapanese::False,
    }
}

/// Convert the katakana from the input string to hiragana
fn convert_katakana(text: &str) -> String {
    let mut buf = String::with_capacity(text.len());
    text.chars().for_each(|c| {
        match c as u32 {
            0x30a1..=0x30f6 => buf.push(char::from_u32(c as u32 - (0x30a1 - 0x3041)).unwrap()),
            0x30f7 => buf.push_str("ゔぁ"),
            0x30f8 => buf.push_str("ゔぃ"),
            0x30f9 => buf.push_str("ゔぇ"),
            0x30fa => buf.push_str("ゔぉ"),
            _ => buf.push(c),
        };
    });
    buf
}

/// Convert the hiragana from the input string to latin characters
fn hiragana_to_romaji(text: &str) -> String {
    let mut buf = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    let mut kc_match = None;

    while let Some((i, c)) = chars.peek().copied() {
        if util::is_char_in_range(c, util::HIRAGANA) {
            match kc_match {
                Some((m_i, n_char, m_rom)) => {
                    let kc_str = &text[m_i..i + c.len_utf8()];
                    match hepburn_dict::HEPBURN_DICT.get(kc_str).copied() {
                        Some(rom) => {
                            // If we have reached the maximum key length,
                            // the match can be added directly
                            if n_char >= hepburn_dict::HEPBURN_MAX_KLEN - 1 {
                                buf.push_str(rom);
                                kc_match = None;
                                chars.next();
                            } else {
                                kc_match = Some((m_i, n_char + 1, rom));
                                chars.next();
                            }
                        }
                        None => {
                            // Add the previous match and dont advance the iterator
                            buf.push_str(m_rom);
                            kc_match = None;
                        }
                    }
                }
                None => {
                    let kc_str = &text[i..i + c.len_utf8()];
                    match hepburn_dict::HEPBURN_DICT.get(kc_str).copied() {
                        Some(rom) => {
                            kc_match = Some((i, 1, rom));
                        }
                        None => buf.push(c),
                    }
                    chars.next();
                }
            }
        } else if c == util::PROLONGED_SOUND_MARK {
            if let Some((_, _, rom)) = kc_match {
                buf.push_str(rom);
                kc_match = None;
            }
            buf.push(buf.chars().last().unwrap_or('-'));
            chars.next();
        } else {
            buf.push(c);
            chars.next();
        }
    }

    if let Some((_, _, rom)) = kc_match {
        buf.push_str(rom);
    }

    buf
}

/// Convert the leading kanji from the input string to hiragana
///
/// # Arguments
///
/// * `text` - Input string starting with the kanji to convert.
///
///   The input needs to be NFKC-normalized and synonymous kanji need to be
///   replaced using [`convert_syn`].
///
/// * `btext` - Buffer string (leading kana)
///
/// # Return
///
/// * `0` - String of hiragana
/// * `1` -  Number of converted chars from the input string
fn convert_kanji(text: &str, btext: &str, dict: &PhfMap) -> (String, usize) {
    let mut translation: Option<String> = None;
    let mut i_c = 0;
    let mut n_c = 0;
    let mut char_indices = text.char_indices().peekable();

    while let Some((i, c)) = char_indices.next() {
        let kanji = &text[0..i + c.len_utf8()];
        let mut more_chars = 0;

        let this_tl = match dict.get::<KanjiString, Readings>(KanjiString::new(kanji)) {
            Some(readings) => readings.iter().and_then(|mut ri| {
                ri.find_map(|r| match r {
                    types::Reading::Simple { hira } => Some(hira),
                    types::Reading::Tail { mut hira, ch } => {
                        char_indices.peek().and_then(|(_, next_c)| {
                            // Shortcut if the next character is not hiragana
                            if util::is_char_in_range(*next_c, util::HIRAGANA) {
                                util::CLETTERS.get(&ch).and_then(|cltr| {
                                    if cltr.contains(next_c) {
                                        // Add the next character to the char count
                                        more_chars += 1;
                                        hira.push(*next_c);
                                        Some(hira)
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            }
                        })
                    }
                    types::Reading::Context { hira, ctx } => {
                        if btext.contains(&ctx) {
                            Some(hira)
                        } else {
                            None
                        }
                    }
                })
            }),
            None => {
                break;
            }
        };

        i_c += 1;
        if let Some(tl) = this_tl {
            translation = Some(tl);
            n_c = i_c + more_chars;
        }
    }

    translation.map(|tl| (tl, n_c)).unwrap_or_default()
}

/// NFKC-normalize the text, convert all synonymous kanji
/// and replace iteration marks (`々`)
fn normalize(text: &str) -> String {
    let mut imcount = 0;
    let replacements = text.char_indices().filter_map(|(i, c)| {
        if c == util::ITERATION_MARK {
            // Count iteration marks
            if imcount == 0 {
                imcount = 1;
                for c in text[i + c.len_utf8()..].chars() {
                    if c == util::ITERATION_MARK {
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
            syn_dict::SYN_DICT
                .get(&c)
                .map(|r_char| (i, c.len_utf8(), *r_char))
                .or_else(|| {
                    // Dont normalize japanese punctuation, we need it to add correct spacing
                    if util::is_char_fwidth_punctuation(c) {
                        Some((i, c.len_utf8(), c))
                    } else {
                        None
                    }
                })
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("\u{ff1f}", "?")]
    #[case("\u{ff1e}", ">")]
    #[case("…", "...")]
    #[case("‥", "..")]
    #[case("\u{FF70}", "\u{30FC}")]
    fn t_unicode_nfkc(#[case] text: &str, #[case] expect: &str) {
        let res = text.nfkc().collect::<String>();
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("壱意", "一意")]
    #[case("", "")]
    #[case("Abc", "Abc")]
    fn t_normalize(#[case] text: &str, #[case] expect: &str) {
        let res = normalize(text);
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("ァ", "ぁ")]
    #[case("ヷ", "ゔぁ")]
    #[case("ヸ", "ゔぃ")]
    #[case("ヹ", "ゔぇ")]
    #[case("ヺ", "ゔぉ")]
    #[case("", "")]
    #[case("Abc", "Abc")]
    fn t_convert_katakana(#[case] text: &str, #[case] expect: &str) {
        let res = convert_katakana(text);
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("", "")]
    #[case("Abc", "Abc")]
    #[case("ば", "ba")]
    #[case("ばば", "baba")]
    #[case("ばー", "baa")]
    #[case("っふぁ", "ffa")]
    fn t_to_romaji(#[case] text: &str, #[case] expect: &str) {
        let res = hiragana_to_romaji(text);
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("会っAbc", "あっ", 2)]
    #[case("渋谷", "しぶや", 2)]
    #[case(
        "東北大学電気通信研究所",
        "とうほくだいがくでんきつうしんけんきゅうじょ",
        11
    )]
    #[case("暑中お見舞い申し上げます", "しょちゅうおみまいもうしあげます", 12)]
    fn t_convert_kanji(#[case] text: &str, #[case] expect: &str, #[case] expect_n: usize) {
        let dict = PhfMap::new(util::KANJI_DICT);
        let (res, n) = convert_kanji(text, "", &dict);
        assert_eq!(res, expect);
        assert_eq!(n, expect_n);
    }
}
