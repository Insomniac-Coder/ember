#$ test: run-pass
#$ rules: RNG-8, ERR-4, STD-19
#$ stdout: 5 9 true true 0
# D-419 — `unwrap_or_default` answers through `Default`, as `T.default()`
# does: a range type has none of its own (ODR-093) but may be given one by
# the program, which constructs it; a struct's, `char`'s and an `Option`'s
# are theirs. Each was `E0900`, "not implemented yet".

type Level = i32 in 1 ..= 10

extend Level implements Default:
    fn default() -> Level:
        return 5

struct P:
    x: int

extend P implements Default:
    fn default() -> P:
        return P(x = 9)

fn or_default[T: Default](owned o: Option[T]) -> T:
    return o.unwrap_or_default()

fn main():
    a: Option[Level] = None
    b: Option[P] = None
    c: Option[char] = None
    d: Option[Option[int]] = None
    e: Option[int] = None
    println(a.unwrap_or_default(), b.unwrap_or_default().x, c.unwrap_or_default() == '\0', d.unwrap_or_default().is_none(), or_default(e))
