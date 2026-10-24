# A25 as a Python programmer writes it: words = line.lower().split()
fn count_words(line: str) -> int:
    words: Array[str] = line.to_lower().split_whitespace().collect()
    return words.len()

fn main():
    println(count_words("A b c"))
