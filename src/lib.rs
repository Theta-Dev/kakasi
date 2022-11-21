mod phfbin;
mod syn_dict;
mod types;

pub use types::KakasiResult;

use std::borrow::Cow;

use unicode_normalization::UnicodeNormalization;

use phfbin::PhfMap;
use types::{KanjiString, Readings};

const KANJI_DICT: &[u8] = include_bytes!("./kanji_dict.bin");

static CLETTERS: phf::Map<u8, &[char]> = phf::phf_map!(
    b'a' => &['あ', 'ぁ', 'っ', 'わ', 'ゎ'],
    b'i' => &['い', 'ぃ', 'っ', 'ゐ'],
    b'u' => &['う', 'ぅ', 'っ'],
    b'e' => &['え', 'ぇ', 'っ', 'ゑ'],
    b'o' => &['お', 'ぉ', 'っ'],
    b'k' => &['か', 'ゕ', 'き', 'く', 'け', 'ゖ', 'こ', 'っ'],
    b'g' => &['が', 'ぎ', 'ぐ', 'げ', 'ご', 'っ'],
    b's' => &['さ', 'し', 'す', 'せ', 'そ', 'っ'],
    b'z' => &['ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'っ'],
    b'j' => &['ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'っ'],
    b't' => &['た', 'ち', 'つ', 'て', 'と', 'っ'],
    b'd' => &['だ', 'ぢ', 'づ', 'で', 'ど', 'っ'],
    b'c' => &['ち', 'っ'],
    b'n' => &['な', 'に', 'ぬ', 'ね', 'の', 'ん'],
    b'h' => &['は', 'ひ', 'ふ', 'へ', 'ほ', 'っ'],
    b'b' => &['ば', 'び', 'ぶ', 'べ', 'ぼ', 'っ'],
    b'f' => &['ふ', 'っ'],
    b'p' => &['ぱ', 'ぴ', 'ぷ', 'ぺ', 'ぽ', 'っ'],
    b'm' => &['ま', 'み', 'む', 'め', 'も'],
    b'y' => &['や', 'ゃ', 'ゆ', 'ゅ', 'よ', 'ょ'],
    b'r' => &['ら', 'り', 'る', 'れ', 'ろ'],
    b'w' => &['わ', 'ゐ', 'ゑ', 'ゎ', 'を', 'っ'],
    b'v' => &['ゔ'],
);

const SENTENCE_END: [char; 4] = ['!', '?', '.', '。'];
const ENDMARK: [char; 5] = [')', ']', '>', ',', '、'];
const DASH_SYMBOLS: [char; 4] = ['\u{30FC}', '\u{2015}', '\u{2212}', '\u{FF70}'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharType {
    Kanji,
    Katakana,
    Hiragana,
    Symbol,
    Alpha,
}

pub fn convert(text: &str) -> KakasiResult {
    let dict = PhfMap::new(KANJI_DICT);

    // TODO: char conversion should be done with iterators
    let text = text.nfkc().collect::<String>();
    let text = convert_syn(&text);

    let mut char_indices = text.char_indices();
    let mut kana_text = String::new();
    let mut prev_type = CharType::Kanji;
    let mut capitalize = (false, false);

    let mut hiragana = String::new();
    let mut romaji = String::new();

    let conv_kana_txt = |kana_text: &mut String,
                         hiragana: &mut String,
                         romaji: &mut String,
                         capitalize: &mut (bool, bool)| {
        if !kana_text.is_empty() {
            let h = convert_kana(&kana_text);
            hiragana.push_str(&h);
            let mut r = wana_kana::to_romaji::to_romaji(&h);

            if capitalize.0 {
                let done;
                (r, done) = capitalize_first_c(&r);
                capitalize.0 = !done;
            }

            romaji.push_str(&r);
            romaji.push(' ');
        }
    };

    // output_flag
    // means (output buffer?, output text[i]?, copy to buffer and increment i?)
    // possible (False, True, True), (True, False, False), (True, True, True)
    //          (False, False, True)

    while let Some((i, c)) = char_indices.next() {
        let output_flag = if ENDMARK.contains(&c) {
            (CharType::Symbol, true, true, true, false)
        } else if SENTENCE_END.contains(&c) {
            if !capitalize.1 {
                (romaji, _) = capitalize_first_c(&romaji);
                capitalize.1 = true;
            }

            (CharType::Symbol, true, true, true, true)
        } else if DASH_SYMBOLS.contains(&c) {
            (prev_type, false, false, true, false)
        } else if is_sym(c) {
            if prev_type != CharType::Symbol {
                (CharType::Symbol, true, false, true, false)
            } else {
                (CharType::Symbol, false, true, true, false)
            }
        } else if wana_kana::utils::is_char_katakana(c) {
            (
                CharType::Katakana,
                prev_type != CharType::Katakana,
                false,
                true,
                false,
            )
        } else if wana_kana::utils::is_char_hiragana(c) {
            (
                CharType::Hiragana,
                prev_type != CharType::Hiragana,
                false,
                true,
                false,
            )
        } else if c.is_ascii() {
            (
                CharType::Alpha,
                prev_type != CharType::Alpha,
                false,
                true,
                false,
            )
        } else if wana_kana::utils::is_char_kanji(c) {
            conv_kana_txt(&mut kana_text, &mut hiragana, &mut romaji, &mut capitalize);
            let (t, n) = convert_kanji(&text[i..], &kana_text, &dict);

            if n > 0 {
                kana_text = t;
                for _ in 1..n {
                    char_indices.next();
                }
                (CharType::Kanji, false, false, false, false)
            } else {
                // Unknown kanji
                kana_text.clear();
                // TODO: FOR TESTING
                hiragana.push_str("🯄");
                romaji.push_str("🯄");
                (CharType::Kanji, true, false, false, false)
            }
        } else if matches!(c as u32, 0xf000..=0xfffd | 0x10000..=0x10ffd) {
            // PUA: ignore and drop
            conv_kana_txt(&mut kana_text, &mut hiragana, &mut romaji, &mut capitalize);
            kana_text.clear();
            (prev_type, false, false, false, false)
        } else {
            (prev_type, true, true, true, false)
        };

        prev_type = output_flag.0;

        if output_flag.1 && output_flag.2 {
            kana_text.push(c);
            conv_kana_txt(&mut kana_text, &mut hiragana, &mut romaji, &mut capitalize);
            kana_text.clear()
        } else if output_flag.1 && output_flag.3 {
            conv_kana_txt(&mut kana_text, &mut hiragana, &mut romaji, &mut capitalize);
            kana_text = c.to_string();
        } else if output_flag.3 {
            kana_text.push(c);
        }

        if output_flag.4 {
            capitalize.0 = true;
        }
    }

    // Convert last word
    conv_kana_txt(&mut kana_text, &mut hiragana, &mut romaji, &mut capitalize);
    // Remove trailing space
    romaji.pop();

    KakasiResult { hiragana, romaji }
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
    let mut translation = None;
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
                            if wana_kana::utils::is_char_hiragana(*next_c) {
                                CLETTERS.get(&ch).and_then(|cltr| {
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
            None => break,
        };

        i_c += 1;
        if let Some(tl) = this_tl {
            translation = Some(tl);
            n_c = i_c + more_chars;
        }
    }

    translation
        .map(|tl| (tl.to_owned(), n_c))
        .unwrap_or_default()
}

/// Convert all synonymous kanji
///
/// The input text needs to be NFKC-normalized.
fn convert_syn(text: &str) -> Cow<str> {
    let mut replacements = text
        .char_indices()
        .filter_map(|(i, c)| {
            syn_dict::SYN_DICT
                .get(&c)
                .map(|r_char| (i, c.len_utf8(), *r_char))
        })
        .peekable();

    if replacements.peek().is_none() {
        return Cow::Borrowed(text);
    }

    let mut new = String::with_capacity(text.len());
    let mut last = 0;

    for (i, clen, r_char) in replacements {
        new.push_str(&text[last..i]);
        new.push(r_char);
        last = i + clen;
    }
    new.push_str(&text[last..]);
    Cow::Owned(new)
}

fn capitalize_first_c(text: &str) -> (String, bool) {
    let mut done = false;
    let res = text
        .chars()
        .map(|c| {
            if !done && c.is_alphanumeric() {
                done = true;
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect::<String>();
    (res, done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("\u{ff1f}", "?")]
    #[case("\u{ff1e}", ">")]
    fn t_normalize(#[case] text: &str, #[case] expect: &str) {
        let res = text.nfkc().collect::<String>();
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("Abc", "Abc")]
    #[case("壱意", "一意")]
    fn t_convert_syn(#[case] text: &str, #[case] expect: &str) {
        let res = convert_syn(text);
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
        let dict = PhfMap::new(KANJI_DICT);
        let (res, n) = convert_kanji(text, "", &dict);
        assert_eq!(res, expect);
        assert_eq!(n, expect_n);
    }

    #[rstest]
    #[case("", "", "")]
    #[case("構成", "こうせい", "kousei")]
    #[case("好き", "すき", "suki")]
    #[case("大きい", "おおきい", "ookii")]
    #[case("かんたん", "かんたん", "kantan")]
    #[case("にゃ", "にゃ", "nya")]
    #[case("っき", "っき", "kki")]
    #[case("っふぁ", "っふぁ", "ffua")] // "ffa"
    #[case("キャ", "きゃ", "kya")]
    #[case("キュ", "きゅ", "kyu")]
    #[case("キョ", "きょ", "kyo")]
    #[case("。", "。", ".")]
    #[case(
        "漢字とひらがな交じり文",
        "かんじとひらがなまじりぶん",
        "kanji tohiragana majiri bun"
    )]
    #[case(
        "Alphabet 123 and 漢字",
        "Alphabet 123 and かんじ",
        "Alphabet 123 and  kanji"
    )] // TODO: double space
    #[case("日経新聞", "にっけいしんぶん", "nikkei shinbun")]
    #[case("日本国民は、", "にほんこくみんは、", "nihonkokumin ha,")]
    #[case(
        "私がこの子を助けなきゃいけないってことだよね",
        "わたしがこのこをたすけなきゃいけないってことだよね",
        "watashi gakono ko wo tasuke nakyaikenaittekotodayone"
    )]
    #[case("やったー", "やったー", "yatta-")]
    #[case("でっでー", "でっでー", "dedde-")]
    #[case("てんさーふろー", "てんさーふろー", "tensa-furo-")]
    #[case("オレンジ色", "おれんじいろ", "orenji iro")]
    #[case("檸檬は、レモン色", "れもんは、れもんいろ", "remon ha, remon iro")]
    fn romanize(#[case] text: &str, #[case] hiragana: &str, #[case] romaji: &str) {
        let res = convert(text);
        assert_eq!(res.hiragana, hiragana);
        assert_eq!(res.romaji, romaji);
    }
}
