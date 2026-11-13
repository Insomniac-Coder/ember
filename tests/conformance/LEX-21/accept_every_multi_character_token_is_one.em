#$ test: run-pass
#$ rules: LEX-21
#$ stdout: 8 8 3 4 6 Some(4) None
## `[LEX-21]` - tokenisation is maximal munch: `**=` before `**`, `..=` before `..`, `?.` is one token,
## `//=` before `//` before `/` (the last in the sibling test).

struct Box:
    v: int

fn pick(b: Option[Box]) -> Option[int]:
    return b?.v

fn main():
    x = 2
    x **= 3
    y = 2 ** 3
    r1 = 0..3
    r2 = 0..=3
    s = 0
    for i in 0..=3:
        s += i
    println(x, y, r1.len(), r2.len(), s, pick(Some(Box(4))), pick(None))
