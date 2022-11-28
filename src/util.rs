use crate::types::CharType;

pub const KANJI_DICT: &[u8] = include_bytes!("./kanji_dict.bin");

pub static CLETTERS: phf::Map<u8, &[char]> = phf::phf_map!(
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

pub static PCT_DICT: phf::Map<char, (char, CharType)> = phf::phf_map!(
    '、' => (',', CharType::TrailingPunct),
    '。' => ('.', CharType::TrailingPunct),
    '「' => ('"', CharType::LeadingPunct),
    '」' => ('"', CharType::TrailingPunct),
    '『' => ('"', CharType::LeadingPunct),
    '』' => ('"', CharType::TrailingPunct),
    '〜' => ('~', CharType::JoiningPunct),
    '・' => (' ', CharType::Whitespace),
    '〈' => ('<', CharType::LeadingPunct),
    '〉' => ('>', CharType::TrailingPunct),
    '《' => ('«', CharType::LeadingPunct),
    '》' => ('»', CharType::TrailingPunct),
    '【' => ('[', CharType::LeadingPunct),
    '】' => (']', CharType::TrailingPunct),
    '〔' => ('(', CharType::LeadingPunct),
    '〕' => (')', CharType::TrailingPunct),
    '〖' => ('[', CharType::LeadingPunct),
    '〗' => (']', CharType::TrailingPunct),
    '〘' => ('(', CharType::LeadingPunct),
    '〙' => (')', CharType::TrailingPunct),
    '〝' => ('"', CharType::LeadingPunct),
    '〟' => ('"', CharType::TrailingPunct),
    '：' => (':', CharType::TrailingPunct),
    '；' => (';', CharType::TrailingPunct),
    '！' => ('!', CharType::TrailingPunct),
    '？' => ('?', CharType::TrailingPunct),
    '＃' => ('?', CharType::LeadingPunct),
    '）' => (')', CharType::TrailingPunct),
    '］' => (']', CharType::TrailingPunct),
    '｝' => ('}', CharType::TrailingPunct),
    '（' => ('(', CharType::LeadingPunct),
    '［' => ('[', CharType::LeadingPunct),
    '｛' => ('{', CharType::LeadingPunct),
    '＿' => ('{', CharType::JoiningPunct),

    ':' => (':', CharType::TrailingPunct),
    ';' => (';', CharType::TrailingPunct),
    '!' => ('!', CharType::TrailingPunct),
    '?' => ('?', CharType::TrailingPunct),
    '#' => ('#', CharType::JoiningPunct),
    ')' => (')', CharType::TrailingPunct),
    ']' => (']', CharType::TrailingPunct),
    '}' => ('}', CharType::TrailingPunct),
    '(' => ('(', CharType::LeadingPunct),
    '[' => ('[', CharType::LeadingPunct),
    '{' => ('{', CharType::LeadingPunct),
    '_' => ('_', CharType::JoiningPunct),
);

pub const HIRAGANA: (u32, u32) = (0x3041, 0x3096);
pub const KATAKANA: (u32, u32) = (0x30A1, 0x30FA);
pub const KANJI: (u32, u32) = (0x4E00, 0x9FAF);

pub const ITERATION_MARK: char = '々';
pub const PROLONGED_SOUND_MARK: char = 'ー';

const CJK_SYMBOLS_PUNCTUATION: (u32, u32) = (0x3000, 0x303F);
const KANA_PUNCTUATION: (u32, u32) = (0xFF61, 0xFF65);
const KATAKANA_PUNCTUATION: (u32, u32) = (0x30FB, 0x30FC);
pub const ZENKAKU_PUNCTUATION_1: (u32, u32) = (0xFF01, 0xFF0F);
pub const ZENKAKU_PUNCTUATION_2: (u32, u32) = (0xFF1A, 0xFF1F);
pub const ZENKAKU_PUNCTUATION_3: (u32, u32) = (0xFF3B, 0xFF3F);
pub const ZENKAKU_PUNCTUATION_4: (u32, u32) = (0xFF5B, 0xFF60);

const JA_PUNCTUATION_RANGES: [(u32, u32); 7] = [
    CJK_SYMBOLS_PUNCTUATION,
    KANA_PUNCTUATION,
    KATAKANA_PUNCTUATION,
    ZENKAKU_PUNCTUATION_1,
    ZENKAKU_PUNCTUATION_2,
    ZENKAKU_PUNCTUATION_3,
    ZENKAKU_PUNCTUATION_4,
];

const FW_PUNCTUATION_RANGES: [(u32, u32); 4] = [
    ZENKAKU_PUNCTUATION_1,
    ZENKAKU_PUNCTUATION_2,
    ZENKAKU_PUNCTUATION_3,
    ZENKAKU_PUNCTUATION_4,
];

pub fn is_char_in_range(c: char, range: (u32, u32)) -> bool {
    range.0 <= c as u32 && c as u32 <= range.1
}

pub fn is_char_japanese_punctuation(c: char) -> bool {
    JA_PUNCTUATION_RANGES
        .iter()
        .any(|r| is_char_in_range(c, *r))
}

pub fn is_char_fwidth_punctuation(c: char) -> bool {
    FW_PUNCTUATION_RANGES
        .iter()
        .any(|r| is_char_in_range(c, *r))
}

pub fn capitalize_first_c(text: &str) -> String {
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
    res
}

pub fn ensure_trailing_space(text: &mut String, ts: bool) {
    if text.is_empty() || text.ends_with('\n') {
        return;
    }

    if text.ends_with(' ') {
        if !ts {
            text.pop();
        }
    } else if ts {
        text.push(' ');
    }
}
