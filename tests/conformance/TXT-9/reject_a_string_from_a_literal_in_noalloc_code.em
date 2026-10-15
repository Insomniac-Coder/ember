#$ test: compile-fail
#$ rules: TXT-9
#$ error[E4001]: `@noalloc` function `named` reaches an allocation
# `[TXT-9]` — a string literal where a `String` is expected makes a new `String`, and the
# allocation carries `Alloc` at the literal.

@noalloc
fn named() -> int:
    s: String = "text"
    return s.len()

fn main():
    println(named())
