#$ test: run-pass
#$ rules: STD-7
#$ stdout: 12 Some(-5) Some(5) Some(-4)
#$ stdout: true 2
#$ stdout: false 3 a
#$ stdout: Some('c') 2
#$ stdout: true true false true false 2
#$ stdout: true false 1
## `[STD-7]` — a `Deque[T]` pushes and pops at both ends; a `SmallArray[T, N]` holds `N` values
## inline and moves to the heap with the next; a `BitSet` holds non-negative integers, a bit each.

from std.collections import BitSet, Deque, SmallArray

fn main():
    d = Deque[int].new()
    for i in 0..6:
        d.push_back(i)
        d.push_front(-i)
    println(d.len(), d.pop_front(), d.pop_back(), d.get(0))

    s = SmallArray[String, 2].new()
    s.push(String.from("a"))
    s.push(String.from("b"))
    println(s.is_inline(), s.len())
    s.push(String.from("c"))
    if Some(first) = s.get(0):
        println(s.is_inline(), s.len(), first)
    println(s.pop(), s.len())

    b = BitSet.new()
    println(b.insert(3), b.insert(200), b.insert(3), b.contains(200), b.contains(4), b.len())
    println(b.remove(3), b.remove(3), b.len())
