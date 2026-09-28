#$ test: run-pass
#$ rules: STD-19
#$ profiles: debug, release, shipping
#$ stdout:
#$ Some(5) Some(1) None
#$ Some(2) Some(5)
#$ Some(120) Some(1) None
#$ Some(9) Some(1) Some(9)
#$ Some('apple') Some('apple') Some('fig')
# `[STD-19]` — `max`, `min`, `max_by_key`, `min_by_key`, `max_by`, `min_by`
# and `reduce`, on a program's own iterator, a list's view iterator (whose
# items are references) and one of strings. Among equal items `max` gives the
# last and `min` the first; an empty iterator gives `None`.

from std.core import Iterator, Ordering

struct Count:
    n: int
    limit: int

extend Count implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        if self.n >= self.limit:
            return None
        self.n += 1
        return Some(self.n)

fn main():
    println(Count(0, 5).max(), Count(0, 5).min(), Count(0, 0).max())
    println(Count(0, 5).max_by_key(fn(x) => (x * 7) % 5), Count(0, 5).min_by_key(fn(x) => (x * 7) % 5))
    println(Count(0, 5).reduce(fn(a, b) => a * b), Count(0, 5).max_by(fn(a, b) => b.cmp(a)), Count(0, 0).reduce(fn(a, b) => a + b))
    xs: Array[int] = [4, 9, 2, 9, 1]
    println(xs.iter().max(), xs.iter().min(), xs.iter().min_by_key(fn(x) => -x))
    words: Array[String] = [String.from("pear"), String.from("fig"), String.from("apple")]
    println(words.iter().max_by_key(fn(w) => w.len()), words.iter().min(), words.iter().min_by(fn(a, b) => a.len().cmp(b.len())))
