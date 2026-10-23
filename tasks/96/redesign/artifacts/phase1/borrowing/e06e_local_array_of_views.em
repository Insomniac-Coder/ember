# A6/A22: the same in Ember: a local Array of views into a local String.
fn count_words(line: str) -> int:
    lower = line.to_lower()
    words: Array[str] = []
    for w in lower.split_whitespace():
        words.push(w)
    return words.len()

fn main():
    println(count_words("A b"))
