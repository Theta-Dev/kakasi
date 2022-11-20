fn main() {
    for line in std::io::stdin().lines() {
        let res = kakasi::convert(&line.unwrap());
        println!("{} - {}", res.hiragana, res.romaji);
    }
}
