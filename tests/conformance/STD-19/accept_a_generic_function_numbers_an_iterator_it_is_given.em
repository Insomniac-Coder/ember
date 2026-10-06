#$ test: run-pass
#$ rules: STD-19, IFC-3, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 18 10
#$ stdout: true true
# `[STD-19]` (D-532) — a function written for any iterator numbers the one it is given:
# `it.enumerate(start=s)` numbers in `I.Position`, which `Iterator` declares an `ItemCount`, and
# an `ItemCount` is `Ord` and `Copy` too (`[IFC-3]`), so `Enumerate`'s `next` (which needs them)
# is there for a `for` and for a call, and two numbers compare and copy. It was refused: an
# associated type's bound did not bring its parents.

fn numbered[I: Iterator](owned it: I, start: int) -> int:
    total = 0
    for i, _x in it.enumerate(start=start):
        total += i.to_int()
    return total

fn first_two_in_order[I: Iterator](owned it: I, start: int) -> bool:
    e = it.enumerate(start=start)
    match e.next():
        Some((a, _x)):
            match e.next():
                Some((b, _y)):
                    c = a
                    return c < b and a < b
                None:
                    return false
        None:
            return false

fn main():
    ys = [1, 2, 3]
    println(numbered(ys.iter(), 5), numbered((0 .. 4).iter(), 1))
    println(first_two_in_order(ys.iter(), 5), first_two_in_order((0u64 .. 10u64).iter(), 3))
