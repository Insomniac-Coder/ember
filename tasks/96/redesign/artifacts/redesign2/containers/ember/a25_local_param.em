# A25: a local list of views of a parameter, not returned.
fn count_words(text: str) -> int:
    words: Array[str] = text.split_whitespace().collect()
    return words.len()

fn main():
    println(count_words("a b c"))
