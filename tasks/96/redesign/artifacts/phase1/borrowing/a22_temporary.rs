// A22: a reference into a temporary that is dropped at the end of the statement.
pub fn first_word_len(line: &str) -> usize {
    let word = line.to_lowercase().split(' ').next().unwrap_or("");
    word.len()
}
