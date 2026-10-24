// A25 returned from a function: the views of a local.
pub fn words_of(line: &str) -> Vec<&str> {
    let lower = line.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    words
}
