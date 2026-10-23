// A22 workaround: bind the temporary to a name (rustc's own suggestion).
pub fn first_word_len(line: &str) -> usize {
    let lower = line.to_lowercase();
    let word = lower.split(' ').next().unwrap_or("");
    word.len()
}
