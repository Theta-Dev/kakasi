fn main() {
    for line in std::io::stdin().lines() {
        println!("{}", kakasi::convert(&line.unwrap()));
    }
}
