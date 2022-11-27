use std::borrow::Cow;

use phf::PhfHash;

use crate::{phfbin::Decodable, util};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KanjiString<'a>(pub Cow<'a, str>);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Readings(&'static [u8]);

pub struct ReadingsIter {
    data: &'static [u8],
    i: usize,
}

/// Reading from the kanji dictionary
#[derive(Debug)]
pub enum Reading {
    /// Default reading
    Simple {
        hira: String,
    },
    ///
    Tail {
        hira: String,
        ch: u8,
    },
    // Reading that has to be preceded by a given context
    Context {
        hira: String,
        ctx: String,
    },
}

/// Character type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharType {
    /// Hiragana (Japanese syllabic characters)
    Hiragana,
    /// Katakana (Japanese syllabic characters)
    Katakana,
    /// Kanji (Japanese ideographic characters)
    Kanji,
    /// Whitespace characters (spaces, newlines, etc.)
    Whitespace,
    /// Japanese punctuation that is led by a space (e.g. `A #B`, `A (B`)
    LeadingPunct,
    /// Japanese punctuation that is followed by a space (e.g. `A: B`, `A) B`)
    TrailingPunct,
    /// Japanese punctuation the is not seperated by spaces (e.g `A~B`, `A_B`)
    JoiningPunct,
    /// Numbers as well as dots and commas used as decimal seperators
    Numeric,
    /// The rest: chars and punctuation for other scripts
    Other,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct KakasiResult {
    pub hiragana: String,
    pub romaji: String,
}

impl<'a> KanjiString<'a> {
    pub fn new(s: &'a str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl Decodable for KanjiString<'_> {
    fn decode(data: &'static [u8]) -> Self {
        KanjiString(Cow::Owned(
            data.chunks_exact(2)
                .map(|c| unsafe { char::from_u32_unchecked(((c[0] as u32) << 8) | c[1] as u32) })
                .collect(),
        ))
    }
}

impl PhfHash for KanjiString<'_> {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.phf_hash(state)
    }
}

impl Decodable for Readings {
    fn decode(data: &'static [u8]) -> Self {
        Self(data)
    }
}

impl Readings {
    pub fn iter(&self) -> Option<ReadingsIter> {
        if self.0.is_empty() {
            None
        } else {
            Some(ReadingsIter { data: self.0, i: 0 })
        }
    }
}

impl Iterator for ReadingsIter {
    type Item = Reading;

    fn next(&mut self) -> Option<Self::Item> {
        let mut hira = String::new();
        let mut ctx = String::new();
        let mut tail: Option<u8> = None;
        let mut read_ctx = false;

        while self.i < self.data.len() {
            let c = &self.data[self.i];
            self.i += 1;

            if c & 0x80 > 0 {
                // Control character
                match c {
                    0x80 => {
                        // Separator between reading and context
                        read_ctx = true;
                    }
                    0xff => {
                        // Separator between readings; output this reading
                        break;
                    }
                    _ => {
                        // Tail
                        tail = Some(*c & 0x7f);
                    }
                }
            } else {
                // Hiragana
                let h = match c {
                    0x7f => ' ',
                    _ => (util::HIRAGANA.0 + *c as u32).try_into().unwrap(),
                };
                if read_ctx {
                    ctx.push(h);
                } else {
                    hira.push(h);
                }
            }
        }

        if hira.is_empty() {
            return None;
        }

        Some(match tail {
            Some(tail) => Reading::Tail { hira, ch: tail },
            None => {
                if !ctx.is_empty() {
                    Reading::Context { hira, ctx }
                } else {
                    Reading::Simple { hira }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::phfbin::PhfMap;

    use super::{KanjiString, Readings};

    #[test]
    fn readings_iter() {
        let dict = PhfMap::new(crate::util::KANJI_DICT);
        let readings = dict
            .get::<KanjiString, Readings>(KanjiString::new("会"))
            .unwrap();

        let res = readings.iter().unwrap().collect::<Vec<_>>();
        dbg!(&res);
    }
}
