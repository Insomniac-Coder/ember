// A6/A22: a local collection of views into a local String: Rust accepts it.
pub fn count_words(line: &str) -> usize {
    let lower = line.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    words.len()
}
