fn main() {
    let mut txt = String::new();
    for line in std::io::stdin().lines().flatten() {
        txt.push_str(&line);
        txt.push('\n');
    }
    let res = kakasi::convert(&txt);
    println!("{}", res.romaji);
}
