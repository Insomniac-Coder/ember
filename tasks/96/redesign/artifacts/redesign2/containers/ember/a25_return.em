# A25 escaping: the list of words is returned (Python: `return line.lower().split()`).
fn words_of(line: str) -> Array[str]:
    lower = line.to_lower()
    words: Array[str] = lower.split_whitespace().collect()
    return words

fn main():
    println(words_of("A b c").len())
