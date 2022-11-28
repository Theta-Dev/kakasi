# kakasi

`kakasi` is a Rust library to transliterate *hiragana*, *katakana* and *kanji* (Japanese text) into *rōmaji* (Latin/Roman alphabet).

It was ported from the [pykakasi](https://codeberg.org/miurahr/pykakasi) library which itself is a port of the original
[kakasi](http://kakasi.namazu.org/) library written in C.

## Usage

Transliterate:

```rust
let res = kakasi::convert("Hello 日本!");
assert_eq!(res.hiragana, "Hello にほん!");
assert_eq!(res.romaji, "Hello nihon !");
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
$ kakasi Hello 日本!
Hello nihon !

## Convert to hiragana
$ kakasi -k Hello 日本!
Hello にほん!

## Read from file
$ kakasi -f rust_article.txt

## Read from STDIN
$ echo "Hello 日本" | kakasi
```

## Performance

Time to convert a 100KB test file using the CLI:

**CPU:** AMD Ryzen 7 5700G

**Test commands:**
```sh
time cat 100K.txt | kakasi-rs > /dev/null
time cat 100K.txt | kakasi -i utf-8 -Ka -Ha -Ja -Sa -s > /dev/null
time cat 100K.txt | python bin/kakasi -Ka -Ha -Ja -Sa -s > /dev/null
```

- kakasi-rs: 9ms (11.11 MB/s)
- [kakasi (C)](https://github.com/loretoparisi/kakasi/): 34ms (2,94 MB/s)
- [pykakasi](https://codeberg.org/miurahr/pykakasi): 796ms (0,13 MB/s)

Programmatic usage:

- **Time:** 6.2056 ms
- **Speed:** 16,11 MB/s
