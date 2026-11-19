#$ test: run-pass
#$ rules: EXP-9, TYP-23
#$ profiles: debug, development, release
#$ stdout: true false false true
#$ true false false true
#$ false true
#$ true false false true true
#$ true false true
# `[EXP-9]` — `x is None` and `x is not None` test an `Option` for absence
# and presence, with `None` on either side, through a reference too. `a is b`
# compares two class handles, or two references: the same place, not equal
# values. A reference to a handle compares the handles.

class Node:
    value: int

@derive(Copy)
struct Pair:
    a: int
    b: int

fn find(xs: Span[int], wanted: int) -> Option[int]:
    for i in range(len(xs)):
        if xs[i] == wanted:
            return Some(i)
    return None

fn empty(ref o: Option[int]) -> bool:
    return o is None

fn main():
    nothing: Option[int] = None
    three: Option[int] = Some(3)
    println(nothing is None, nothing is not None, three is None, three is not None)
    xs = [4, 5, 6]
    println(find(xs.as_span(), 5) is not None, find(xs.as_span(), 9) is not None, None is not nothing, None is nothing)
    println(empty(ref three), empty(ref nothing))
    n = 1
    m = 1
    p = Pair(1, 1)
    r: ref int = ref n
    r1: ref int = ref n
    r2: ref int = ref n
    r3: ref int = ref m
    r4: ref int = ref p.a
    r5: ref int = ref p.b
    println(r1 is r2, r1 is r3, r4 is r5, r is ref n, r is not ref m)
    first = Node(1)
    alias = shares mut first
    other = shares mut Node(1)
    f1: ref shares mut Node = ref first
    f2: ref shares mut Node = ref alias
    f3: ref shares mut Node = ref other
    println(f1 is f2, f1 is f3, first is alias)
