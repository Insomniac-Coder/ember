#$ test: run-pass
#$ rules: TYP-5
#$ stdout: 4
#$ 4
#$ 6
#$ 5
#$ 3
#$ 7
## `[TYP-5]` rule 8 (an `owns T` to `owns dyn I`), rule 7 before any rule (a read through a
## `ref` or an `owns T`), and rule 11 after any one of the others.

interface Weapon:
    fn edge() -> int

struct Sword implements Weapon:
    n: int

    fn edge() -> int:
        return this.n

fn edge_of(w: dyn Weapon) -> int:
    return w.edge()

fn greet(s: StrView) -> int:
    return s.len()

fn total(xs: Span[int]) -> int:
    t = 0
    for x in xs:
        t += x
    return t

fn main():
    sw = Sword(4)
    println(edge_of(ref sw))
    boxed: owns dyn Weapon = owns Sword(4)        # an `owns T` to `owns dyn I`
    println(boxed.edge())
    b: owns Array[int] = owns [1, 2, 3]
    println(total(b))                             # rule 7 then rule 6
    word: String = "hello"
    r: ref String = ref word
    println(greet(r))                             # rule 7 then rule 4
    n = 3
    cap: Option[u64] = n                          # rule 11 after rule 12
    if cap exists as c:
        println(c)
    slot: Option[owns dyn Weapon] = owns Sword(7)  # rule 11 after rule 8
    if slot exists as w:
        println(w.edge())
