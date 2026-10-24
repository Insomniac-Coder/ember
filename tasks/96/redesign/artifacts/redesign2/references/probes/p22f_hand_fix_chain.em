# A22: e22's chain written the way today's checker accepts it: every temporary bound to a name.
fn first_word_len(line: str) -> int:
    lower = line.to_lower()
    parts = lower.split(" ")
    word = parts.next().unwrap_or("")
    return word.len()

fn main():
    println(first_word_len("Hello World"))
