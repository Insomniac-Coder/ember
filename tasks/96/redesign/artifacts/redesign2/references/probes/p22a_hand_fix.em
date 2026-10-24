# A22: the program the extension rule rewrites e22b into (what the E3060 help asks the user to write).
fn first_word_len(line: str) -> int:
    lower = line.to_lower()
    word = lower.trim()
    return word.len()

fn main():
    println(first_word_len(" Hello "))
