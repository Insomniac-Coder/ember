# A22: a view of a temporary returned where the declared result is owned.
fn first_word(line: str) -> String:
    return line.to_lower().trim()

fn main():
    println(first_word(" Hello "))
