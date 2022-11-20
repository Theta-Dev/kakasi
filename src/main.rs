fn main() {
    for line in std::io::stdin().lines() {
        let res = kakasi::convert(&line.unwrap());
        println!("{}\n{}\n\n", res.hiragana, res.romaji);
    }
}
