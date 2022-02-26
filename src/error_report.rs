pub fn error_at(input: &str, pos: usize, msg: &str) {
    eprintln!("{}", input);
    eprintln!("{}^ {}", " ".repeat(pos), msg);
}
