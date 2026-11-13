# Clause BRW-12.5: "A call whose result is a new value the callee made (a list it built, a list of
# views included) hands that value to the name, which owns it (`[LT-45]` bounds the life of a value
# that holds views)."
#$ test: run-pass
#$ rules: BRW-12, LT-45, OWN-9
#$ stdout: 4
#$ stdout: 3
# Each name owns its list and may change it; a read-only view could not (`[BRW-15]`). `letters`'s list
# points into `s`, its one source parameter (`[LT-1]` rule 2).

fn build(n: int) -> Array[int]:
    xs: Array[int] = []
    for i in 0..n:
        xs.push(owns i)
    return xs

fn letters(s: StrView) -> Array[StrView]:
    return [s[0..1], s[1..2]]

fn main():
    xs = build(3)
    xs.push(owns 9)
    println(xs.len())
    text = String("ab")
    ws = letters(text)
    ws.push(owns "z")
    println(ws.len())
