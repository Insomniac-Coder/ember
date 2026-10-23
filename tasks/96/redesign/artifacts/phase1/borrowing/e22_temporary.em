# A22: a view into a temporary string kept past the statement.
fn first_word_len(line: str) -> int:
    word = line.to_lower().split(" ").next().unwrap_or("")
    return word.len()

fn main():
    println(first_word_len("Hello World"))
