#$ test: run-pass
#$ rules: LT-1a, LT-1
#$ stdout: bb bb
#$ stdout: 5
# `[LT-1a]` — "`self` names the receiver": `@borrows(self, other)` says the
# result may point into either, in a plain type's method and a generic one's.

struct Name:
    text: String

    fn longer(other: Name) -> ref String:
        if other.text.len() > this.text.len():
            return ref other.text
        return ref this.text

[T]
struct Cell2:
    value: T

    fn get() -> ref T:
        return ref this.value

fn main():
    a = Name("a")
    b = Name("bb")
    r = a.longer(b)
    s = b.longer(a)
    println(r, s)
    c = Cell2(5)
    println(c.get())
