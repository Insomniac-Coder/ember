# A25 with the temporary named first (Rust's own fix for E0716): still refused today.
fn count_words(line: str) -> int:
    lower = line.to_lower()
    words: Array[str] = lower.split_whitespace().collect()
    return words.len()

fn main():
    println(count_words("A b c"))
