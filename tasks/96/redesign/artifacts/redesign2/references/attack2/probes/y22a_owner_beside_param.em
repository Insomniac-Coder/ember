# Attack on A22 6.4 ("the owner follows the view"): the outer variable is a `mut` parameter. Its
# storage is the caller's, so no hidden owner can sit beside it. Today: what is the error?
fn pick(mut best: str, lines: Array[String]):
    for line in lines:
        w = line.to_lower().trim()
        if w.len() > best.len():
            best = w

fn main():
    b: str = ""
    pick(b, ["Hello", "World!"])
    println(b)
