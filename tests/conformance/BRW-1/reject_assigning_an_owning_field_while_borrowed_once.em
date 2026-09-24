#$ test: compile-fail
#$ rules: BRW-1, DIA-14
# Assigning a field that owns storage drops the old value and writes the new
# one at one expression: while a reference into the struct lives that is one
# mistake, reported once.

struct Q:
    arr: Array[int]

fn first(mut h: Q) -> ref mut int:
    return ref mut h.arr[0]

fn main():
    a: Array[int] = Array[int]()
    a.push(1)
    h = Q(arr=a)
    r = first(h)
    h.arr = Array[int]()    #$ error[E3021]: `h` cannot be written while it is borrowed
    println(r)
