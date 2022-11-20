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

pub fn convert(text: &str) -> KakasiResult {
    let dict = PhfMap::new(KANJI_DICT);

    // TODO: char conversion should be done with iterators
    let text = text.nfkc().collect::<String>();
    let text = convert_syn(&text);

    let hiragana = convert_kanji(&text, "", &dict).0;
    let romaji = wana_kana::to_romaji::to_romaji(&hiragana);

    KakasiResult { hiragana, romaji }
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
/// * `btext` -
///
/// # Return
///
/// * `0` - String of hiragana
/// * `1` -  Number of converted chars from the input string
fn convert_kanji(text: &str, btext: &str, dict: &PhfMap) -> (String, usize) {
    let mut translation = None;
    let mut n_c = 0;
    let mut char_indices = text.char_indices().peekable();

    while let Some((i, c)) = char_indices.next() {
        let kanji = &text[0..i + c.len_utf8()];

        let this_tl = dict
            .get::<KanjiString, Readings>(KanjiString::new(kanji))
            .and_then(|readings| {
                readings.iter().find_map(|r| match r {
                    types::Reading::Simple { hira } => Some(hira),
                    types::Reading::Tail { mut hira, ch } => {
                        char_indices.peek().and_then(|(_, next_c)| {
                            // Shortcut if the next character is not hiragana
                            if wana_kana::utils::is_char_hiragana(*next_c) {
                                CLETTERS.get(&ch).and_then(|cltr| {
                                    if cltr.contains(next_c) {
                                        // Add the next character to the char count
                                        n_c += 1;
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
            });

        match this_tl {
            Some(this_tl) => translation = Some(this_tl),
            None => break,
        }
        n_c += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Abc", "Abc")]
    #[case("壱意", "一意")]
    fn t_convert_syn(#[case] text: &str, #[case] expect: &str) {
        let res = convert_syn(text);
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("会っAbc", "あっ", 2)]
    fn t_convert_kanji(#[case] text: &str, #[case] expect: &str, #[case] expect_n: usize) {
        let dict = PhfMap::new(KANJI_DICT);
        let (res, n) = convert_kanji(text, "", &dict);
        assert_eq!(res, expect);
        assert_eq!(n, expect_n);
    }
}
