#$ test: run-pass
#$ rules: LT-20, LT-1
#$ profiles: debug, release
#$ stdout: Some('a') Some('b')
#$ stdout: Some('c') 3 Some('b')
# `[LT-20]`, `[LT-1]` — an iterator that is a `Copy` view gives views of the
# text it holds. `mut self` reaches it through a reference for its mode only
# (a source of the first kind), so what `next` returns borrows the text, not
# the iterator: two items live at once, and std's `Iterator` defaults keep
# one past the next call (D-470).

@derive(Copy)
struct Words:
    rest: str

extend Words implements Iterator:
    type Item = str

    fn next(mut self) -> Option[str]:
        rest = self.rest
        if rest.len() == 0:
            return None
        match rest.find(" "):
            Some(at):
                self.rest = rest[at + 1..]
                return Some(rest[..at])
            None:
                self.rest = rest[rest.len()..]
                return Some(rest)

fn main():
    w = Words(rest = "a b c")
    x = w.next()
    y = w.next()
    println(x, y)
    println(Words(rest = "a b c").last(), Words(rest = "a b c").count(), Words(rest = "a b c").nth(1))
