# A22: a view of a temporary String kept past the statement.
fn first_word_len(line: str) -> int:
    word = line.to_lower().trim()
    return word.len()

fn main():
    println(first_word_len(" Hello "))
