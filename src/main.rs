fn main() {
    let mut txt = String::new();
    for line in std::io::stdin().lines() {
        if let Ok(line) = line {
            txt.push_str(&line);
            txt.push('\n');
        }
    }
    let res = kakasi::convert(&txt);
    println!("{}", res.romaji);
}
