use zstd;

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = zstd::Decoder::new(stdin.lock()).unwrap();
    std::io::copy(&mut reader, &mut stdout.lock()).unwrap();
}
