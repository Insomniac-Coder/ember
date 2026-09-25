#$ test: run-pass
#$ rules: EXP-9, TYP-23
#$ profiles: debug, release, shipping
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

fn empty(o: ref Option[int]) -> bool:
    return o is None

fn same(a: ref int, b: ref int) -> bool:
    return a is b

fn same_node(a: ref Node, b: ref Node) -> bool:
    return a is b

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
    println(same(ref n, ref n), same(ref n, ref m), same(ref p.a, ref p.b), r is ref n, r is not ref m)
    first = Node(1)
    alias = first
    other = Node(1)
    println(same_node(ref first, ref alias), same_node(ref first, ref other), first is alias)
