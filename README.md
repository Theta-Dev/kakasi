# kakasi

[![crates.io](https://img.shields.io/crates/v/kakasi)](https://crates.io/crates/kakasi)
[![docs.rs](https://img.shields.io/docsrs/kakasi)](https://docs.rs/kakasi)
[![licence](https://img.shields.io/crates/l/kakasi)](https://github.com/Theta-Dev/kakasi)

`kakasi` is a Rust library to transliterate _hiragana_, _katakana_ and _kanji_ (Japanese text) into _rōmaji_ (Latin/Roman alphabet).

It was ported from the [pykakasi](https://codeberg.org/miurahr/pykakasi) library which itself is a port of the original
[kakasi](http://kakasi.namazu.org/) library written in C.

## Usage

Transliterate:

```rust
let res = kakasi::convert("こんにちは世界!");
assert_eq!(res.hiragana, "こんにちはせかい!");
assert_eq!(res.romaji, "konnichiha sekai!");
```

Check if a string contains Japanese characters:

```rust
use kakasi::IsJapanese;

assert_eq!(kakasi::is_japanese("Abc"), IsJapanese::False);
assert_eq!(kakasi::is_japanese("日本"), IsJapanese::Maybe);
assert_eq!(kakasi::is_japanese("ラスト"), IsJapanese::True);
```

## CLI

```sh
$ cargo install kakasi

## Convert to romaji
$ kakasi こんにちは世界!
konnichiha sekai!

## Convert to hiragana
$ kakasi -k こんにちは世界!
こんにちはせかい!

## Read from file
$ kakasi -f rust_article.txt

## Read from STDIN
$ echo "こんにちは世界!" | kakasi
```

## Performance

**CPU:** AMD Ryzen 7 5700G

| Text                             | Conversion time | Speed      |
| -------------------------------- | --------------- | ---------- |
| Sentence (161 B)                 | 7.0911 µs       | 22.70 MB/s |
| Rust wikipedia article (31705 B) | 1.5055 ms       | 21.06 MB/s |

### CLI comparison

Time to convert a 100KB test file using the CLI:

| Library                                                    | Time     | Speed      |
| ---------------------------------------------------------- | -------- | ---------- |
| kakasi (Rust)                                              | 7.4 ms   | 13.5 MB/s  |
| [kakasi](https://github.com/loretoparisi/kakasi) (C)       | 33.5 ms  | 2.99 MB/s  |
| [pykakasi](https://codeberg.org/miurahr/pykakasi) (Python) | 810.6 ms | 0.123 MB/s |

**Test commands:**

CLI performance was measured with [hyperfine](https://github.com/sharkdp/hyperfine).

```sh
hyperfine --warmup 3 'cat 100K.txt | kakasi-rs'
hyperfine --warmup 3 'cat 100K.txt | kakasi -i utf-8 -Ka -Ha -Ja -Sa -s'
hyperfine --warmup 3 'cat 100K.txt | python bin/kakasi -Ka -Ha -Ja -Sa -s'
```

## License

kakasi is published under the **GNU GPL-3.0** license.

The Kakasi dictionaries (Files: `codegen/dict/kakasidict.utf8`, `codegen/dict/itajidict.utf8`,
`codegen/dict/hepburn.utf8`)
were taken from the [pykakasi](https://codeberg.org/miurahr/pykakasi) project,
published under the GNU GPL-3.0 license.

**pykakasi**
> Copyright (C) 2010-2021 Hiroshi Miura and contributors(see AUTHORS)

The dictionaries originate from the [kakasi](http://kakasi.namazu.org/) project,
published under the GNU GPL-2.0 license.

**original kakasi**
> Copyright (C) 1992 1993 1994<br>
> Hironobu Takahashi (takahasi@tiny.or.jp),<br>
> Masahiko Sato (masahiko@sato.riec.tohoku.ac.jp),<br>
> Yukiyoshi Kameyama, Miki Inooka, Akihiko Sasaki, Dai Ando, Junichi Okukawa,<br>
> Katsushi Sato and Nobuhiro Yamagishi

For testing I included a copy of the [Japanese Rust wikipedia article](https://ja.wikipedia.org/wiki/Rust_(%E3%83%97%E3%83%AD%E3%82%B0%E3%83%A9%E3%83%9F%E3%83%B3%E3%82%B0%E8%A8%80%E8%AA%9E))
(`tests/rust_article.txt`). The article is published under the Creative Commons
Attribution-ShareAlike License 3.0.
