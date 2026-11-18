#$ test: run-pass
#$ rules: TYP-5
#$ stdout: 200
#$ 200
#$ 3.5
#$ 2
#$ hi
#$ 9
#$ 4
#$ 5
#$ 5
#$ 0
## `[TYP-5]` rules 1, 2, 4, 6, 7, 9, 10, 11 at initialisation, argument, return and field sites.

struct Holder:
    wide: i64
    f: f64

fn widen(x: i16) -> i64:
    return x                         # rule 1 at a return

fn name_len(s: StrView) -> int:
    return s.len()

fn sum(xs: Span[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn maybe(x: Option[int] = None) -> int:
    if x exists as v:
        return v
    return 0

fn pick(flag: bool) -> int:
    if flag:
        return 4
    return panic("no")               # rule 9

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn inc(x: int) -> int:
    return x + 1

fn main():
    a: u8 = 200
    b: u16 = a                       # rule 1: uN to uM
    c: i16 = a                       # rule 1: uN to iM
    h = Holder(c, 1.5)               # rule 1 and a float at field sites
    println(widen(c))
    println(h.wide)
    x: f32 = 1.5
    y: f64 = x + 2.0                 # rule 2
    println(y)
    s: String = "hi"                 # rule 4
    v: StrView = s                   # rule 4: String to StrView
    println(name_len(s))
    println(v)
    arr = [1, 2]
    println(sum(arr) + sum([3, 3]) + b - b)   # rule 6
    println(pick(true))
    println(apply(inc, 4))           # rule 10
    println(maybe(5))                # rule 11
    println(maybe())
