#$ test: run-pass
#$ rules: TYP-1, TYP-27
#$ stdout: [(), (), ()] true 3
#$ stdout: (1, ()) true
#$ stdout: 2 true Some(())
#$ stdout: 7 [(), ()]
# `[TYP-1]` — a `void` value can be anywhere a value can: a fixed array's element, a tuple's
# item, a map's value and a function's parameter (C has no `void` parameter, so it is a byte
# there, as in a struct) (D-355).

fn count(v: void, n: int) -> int:
    return n if v == () else 0

fn main():
    fixed: [(); 3] = [(), (), ()]
    same: [void; 3] = [(), (), ()]
    k = 0
    for u in fixed:
        k += count(u, 1)
    println(fixed, fixed == same, k)
    pair = (1, ())
    println(pair, pair == (1, ()))
    m: Map[int, ()] = {}
    m[3] = ()
    m[4] = ()
    println(len(m), 3 in m, m.get(4))
    units = [(), (), (), ()]
    println(count((), 7), units.as_span()[1..3])
