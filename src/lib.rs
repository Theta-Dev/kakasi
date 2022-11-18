use std::borrow::Cow;

pub fn convert(text: &str) -> String {
    convert_kanji(text, "").0
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
fn convert_kanji(text: &str, btext: &str) -> (String, usize) {
    let mut translation = None;
    let mut n_c = 0;

    for (i, c) in text.char_indices() {
        let kanji = &text[0..i + c.len_utf8()];

        let this_tl = kakasi_dict::lookup_kanji(kanji).and_then(|readings| {
            readings
                .iter()
                .filter_map(|(reading, context)| {
                    if context.is_empty() {
                        Some((reading, false))
                    } else if context.contains(btext) {
                        Some((reading, true))
                    } else {
                        None
                    }
                })
                .max_by_key(|x| x.1)
                .map(|(tl, _)| *tl)
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
        .filter_map(|(i, c)| kakasi_dict::lookup_syn(&c).map(|r_char| (i, c.len_utf8(), *r_char)))
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
        let (res, n) = convert_kanji(text, "");
        assert_eq!(res, expect);
        assert_eq!(n, expect_n);
    }
}
