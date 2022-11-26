mod hepburn_dict;
mod phfbin;
mod syn_dict;
mod types;
mod util;

pub use types::KakasiResult;

use std::borrow::Cow;

use unicode_normalization::UnicodeNormalization;

use phfbin::PhfMap;
use types::{CharType, KanjiString, Readings};

pub fn convert(text: &str) -> KakasiResult {
    let dict = PhfMap::new(util::KANJI_DICT);

    // TODO: char conversion should be done with iterators
    let text = text.nfkc().collect::<String>();
    let text = convert_syn(&text);

    let mut char_indices = text.char_indices().peekable();
    let mut kana_buf = String::new();
    let mut prev_buf_type = CharType::Whitespace;
    let mut prev_acc_type = CharType::Whitespace;
    let mut cap = (false, false);

    let mut res = KakasiResult::default();

    let conv_kana_buf = |kana_buf: &mut String,
                         res: &mut KakasiResult,
                         prev_type: CharType,
                         cap: &mut (bool, bool)| {
        if !kana_buf.is_empty() {
            let hira = convert_katakana(kana_buf);
            res.hiragana.push_str(&hira);
            let mut rom = hiragana_to_romaji(&hira);

            if cap.0 {
                let done;
                (rom, done) = util::capitalize_first_c(&rom);
                cap.0 = !done;

                if !cap.1 {
                    (res.romaji, _) = util::capitalize_first_c(&res.romaji);
                    cap.1 = true;
                }
            }

            util::ensure_trailing_space(
                &mut res.romaji,
                prev_type != CharType::LeadingPunct && prev_type != CharType::JoiningPunct,
            );
            res.romaji.push_str(&rom);

            kana_buf.clear();
        }
    };

    while let Some((i, c)) = char_indices.next() {
        if util::is_char_in_range(c, util::HIRAGANA) {
            if prev_buf_type != CharType::Hiragana {
                conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
            }
            kana_buf.push(c);
            prev_buf_type = CharType::Hiragana;
        } else if util::is_char_in_range(c, util::KATAKANA) {
            if prev_buf_type != CharType::Katakana {
                conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
            }
            kana_buf.push(c);
            prev_buf_type = CharType::Katakana;
        } else if util::is_char_in_range(c, util::KANJI) {
            let (t, n) = convert_kanji(&text[i..], &kana_buf, &dict);
            conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);

            if n > 0 {
                kana_buf = t;
                conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
                for _ in 1..n {
                    char_indices.next();
                }
            } else {
                // Unknown kanji
                // TODO: FOR TESTING
                res.hiragana.push_str("[?]");
                res.romaji.push_str("[?]");
            }
            prev_acc_type = CharType::Kanji;
        } else if c.is_whitespace() {
            conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
            res.hiragana.push(c);
            res.romaji.push(c);
            prev_acc_type = CharType::Whitespace;
        } else if c == '・' {
            conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
            res.hiragana.push(c);
            res.romaji.push(' ');
            prev_acc_type = CharType::Whitespace;
        } else if c == util::PROLONGED_SOUND_MARK {
            if prev_buf_type != CharType::Hiragana && prev_buf_type != CharType::Katakana {
                conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
            }
            kana_buf.push(c);
            prev_buf_type = match prev_buf_type {
                CharType::Hiragana => CharType::Hiragana,
                _ => CharType::Katakana,
            };
        } else {
            conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);

            res.hiragana.push(c);
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

            if (prev_acc_type != CharType::Other && prev_acc_type != CharType::Numeric)
                || util::is_char_japanese_punctuation(c)
            {
                util::ensure_trailing_space(
                    &mut res.romaji,
                    prev_acc_type != CharType::LeadingPunct
                        && prev_acc_type != CharType::JoiningPunct
                        && char_type != CharType::TrailingPunct
                        && char_type != CharType::JoiningPunct,
                );
            }
            res.romaji.push(c_rom);

            if c_rom == '.' && char_type != CharType::Numeric {
                cap.0 = true;
            }

            prev_acc_type = char_type;
        };
    }

    conv_kana_buf(&mut kana_buf, &mut res, prev_acc_type, &mut cap);
    res
}

fn convert_katakana(text: &str) -> String {
    let mut buf = String::new();
    text.chars().for_each(|c| {
        match c as u32 {
            0x30a1..=0x30f6 => {
                char::from_u32(c as u32 - (0x30a1 - 0x3041)).map(|c| buf.push(c));
            }
            0x30f7 => buf.push_str("ゔぁ"),
            0x30f8 => buf.push_str("ゔぃ"),
            0x30f9 => buf.push_str("ゔぇ"),
            0x30fa => buf.push_str("ゔぉ"),
            _ => buf.push(c),
        };
    });
    buf
}

fn hiragana_to_romaji(text: &str) -> String {
    let mut buf = String::new();
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

/// Convert all synonymous kanji and replace iteration characters (`々`)
///
/// The input text needs to be NFKC-normalized.
fn convert_syn(text: &str) -> Cow<str> {
    let mut replacements = text
        .char_indices()
        .filter_map(|(i, c)| {
            syn_dict::SYN_DICT
                .get(&c)
                .map(|r_char| (i, c.len_utf8(), *r_char))
                .or_else(|| {
                    if c == util::ITERATION_MARK {
                        text[0..i]
                            .chars()
                            .last()
                            .map(|prev| (i, c.len_utf8(), prev))
                    } else {
                        None
                    }
                })
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
    #[case("\u{ff1f}", "?")]
    #[case("\u{ff1e}", ">")]
    #[case("…", "...")]
    #[case("‥", "..")]
    #[case("\u{FF70}", "\u{30FC}")]
    fn t_normalize(#[case] text: &str, #[case] expect: &str) {
        let res = text.nfkc().collect::<String>();
        assert_eq!(res, expect);
    }

    #[rstest]
    #[case("壱意", "一意")]
    #[case("", "")]
    #[case("Abc", "Abc")]
    fn t_convert_syn(#[case] text: &str, #[case] expect: &str) {
        let res = convert_syn(text);
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

    #[rstest]
    #[case("", "", "")]
    #[case("構成", "こうせい", "kousei")]
    #[case("好き", "すき", "suki")]
    #[case("大きい", "おおきい", "ookii")]
    #[case("かんたん", "かんたん", "kantan")]
    #[case("にゃ", "にゃ", "nya")]
    #[case("っき", "っき", "kki")]
    #[case("っふぁ", "っふぁ", "ffa")]
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
        "Alphabet 123 and kanji"
    )]
    #[case("日経新聞", "にっけいしんぶん", "nikkei shinbun")]
    #[case("日本国民は、", "にほんこくみんは、", "nihonkokumin ha,")]
    #[case(
        "私がこの子を助けなきゃいけないってことだよね",
        "わたしがこのこをたすけなきゃいけないってことだよね",
        "watashi gakono ko wo tasuke nakyaikenaittekotodayone"
    )]
    #[case("やったー", "やったー", "yattaa")]
    #[case("でっでー", "でっでー", "deddee")]
    #[case("てんさーふろー", "てんさーふろー", "tensaafuroo")]
    #[case("オレンジ色", "おれんじいろ", "orenji iro")]
    #[case("檸檬は、レモン色", "れもんは、れもんいろ", "remon ha, remon iro")]
    #[case("血液1μL", "けつえき1μL", "ketsueki 1μL")]
    #[case("「和風」", "「わふう」", "\"wafuu\"")]
    #[case("て「わ", "て「わ", "te \"wa")]
    #[case("号・雅", "ごう・まさ", "gou masa")]
    #[case("ビーバーが", "びーばーが", "biibaa ga")]
    #[case("ブッシュッー", "ぶっしゅっー", "busshutsuu")]
    #[case("ユーベルヹーク大", "ゆーべるゔぇーくだい", "yuuberuveeku dai")]
    #[case("ヸーヂャニー品", "ゔぃーぢゃにーひん", "viijanii hin")]
    #[case("アヷーリヤ品", "あゔぁーりやひん", "avaariya hin")]
    #[case(
        "安藤 和風（あんどう はるかぜ、慶応2年1月12日（1866年2月26日） - 昭和11年（1936年）12月26日）は、日本のジャーナリスト、マスメディア経営者、俳人、郷土史研究家。通名および俳号は「和風」をそのまま音読みして「わふう」。秋田県の地方紙「秋田魁新報」の事業拡大に貢献し、秋田魁新報社三大柱石の一人と称された。「魁の安藤か、安藤の魁か」と言われるほど、新聞記者としての名声を全国にとどろかせた[4]。",
        "あんどう わふう(あんどう はるかぜ、けいおう2ねん1がつ12にち(1866ねん2がつ26にち) - しょうわ11ねん(1936ねん)12がつ26にち)は、にっぽんのじゃーなりすと、ますめでぃあけいえいしゃ、はいじん、きょうどしけんきゅうか。とおりめいおよびはいごうは「わふう」をそのままおんよみして「わふう」。あきたけんのちほうし「あきたかいしんぽう」のじぎょうかくだいにこうけんし、あきたかいしんぽうしゃさんだいちゅうせきのひとりとしょうされた。「かいのあんどうか、あんどうのかいか」といわれるほど、しんぶんきしゃとしてのめいせいをぜんこくにとどろかせた[4]。",
        "Andou wafuu (andou harukaze, keiou 2 nen 1 gatsu 12 nichi (1866 nen 2 gatsu 26 nichi) - shouwa 11 nen (1936 nen) 12 gatsu 26 nichi) ha, nippon no jaanarisuto, masumedia keieisha, haijin, kyoudoshi kenkyuuka. Toori mei oyobi hai gou ha \"wafuu\" wosonomama onyomi shite \"wafuu\". Akitaken no chihoushi \"akita kai shinpou\" no jigyou kakudai ni kouken shi, akita kai shinpou sha sandai chuuseki no hitori to shousa reta. \"Kai no andou ka, andou no kai ka\" to iwa reruhodo, shinbunkisha toshiteno meisei wo zenkoku nitodorokaseta [4].",
    )]
    #[case(
        "『ザ・トラベルナース』",
        "『ざ・とらべるなーす』",
        "\"za toraberunaasu\""
    )]
    #[case(
        "緑黄色社会『ミチヲユケ』Official Video -「ファーストペンギン！」主題歌",
        "みどりきいろしゃかい『みちをゆけ』Official Video -「ふぁーすとぺんぎん!」しゅだいか",
        "midori kiiro shakai \"michiwoyuke\" Official Video - \"faasutopengin!\" shudaika"
    )]
    #[case(
        "MONKEY MAJIK - Running In The Dark【Lyric Video】（日本語字幕付）",
        "MONKEY MAJIK - Running In The Dark【Lyric Video】(にほんごじまくつき)",
        "MONKEY MAJIK - Running In The Dark [Lyric Video] (nihongo jimaku tsuki)"
    )]
    #[case(
        "取締役第二制作技術部々長",
        "とりしまりやくだいにせいさくぎじゅつぶぶちょう",
        "torishimariyaku daini seisaku gijutsubu buchou"
    )]
    fn romanize(#[case] text: &str, #[case] hiragana: &str, #[case] romaji: &str) {
        let res = convert(text);
        assert_eq!(res.hiragana, hiragana);
        assert_eq!(res.romaji, romaji);
    }
}
