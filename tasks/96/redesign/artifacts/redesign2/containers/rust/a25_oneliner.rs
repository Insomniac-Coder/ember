// A25 as a one-liner in Rust: the list of views of a temporary.
pub fn count_words(line: &str) -> usize {
    let words: Vec<&str> = line.to_lowercase().split_whitespace().collect();
    words.len()
}
