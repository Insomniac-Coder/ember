# A22: a `mut self` method (an iterator's next) called on a temporary, with no view of a temporary kept.
fn first_len(line: str) -> int:
    n = line.split(" ").next().unwrap_or("").len()
    return n

fn main():
    println(first_len("Hello World"))
