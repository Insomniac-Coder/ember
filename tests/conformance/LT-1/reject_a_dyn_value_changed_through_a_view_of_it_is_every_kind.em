#$ test: compile-fail
#$ rules: LT-1, BRW-1
# `[LT-1]` (verdict 151) — a value whose kind may be replaced through a view of it is not followed to
# where it was made: `a` is made a `Mine`, but `mem.swap` (through `ref mut a`) and `change` (through
# its `ref mut` parameter) may make it a `Yours`, whose `pick` gives back the list it was handed, so
# each call points into every argument and the change of `xs` while `r` is used is refused.

import std.mem

interface Picker:
    fn pick(other: Array[int]) -> Span[int]

struct Mine implements Picker:
    items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return this.items

struct Yours implements Picker:
    items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return other

fn change(ref mut q: Box[dyn Picker]):
    q = Box(Yours(items=[9]))

fn swapped():
    a: Box[dyn Picker] = Box(Mine(items=[9]))
    b: Box[dyn Picker] = Box(Yours(items=[9]))
    mem.swap(ref mut a, ref mut b)
    xs = [1, 2]
    r = a.pick(xs)
    xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)

fn changed():
    a: Box[dyn Picker] = Box(Mine(items=[9]))
    change(ref mut a)
    xs = [1, 2]
    r = a.pick(xs)
    xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)

fn main():
    swapped()
    changed()
