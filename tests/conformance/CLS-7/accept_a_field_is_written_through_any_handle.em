#$ test: run-pass
#$ rules: CLS-7, EXC-16, EXC-18
#$ profiles: debug, release
#$ stdout: 1
#$ stdout: [7, 8, 9]
#$ stdout: [8, 9, 10]
# `[CLS-7]` — a class handle has reference semantics: a field is written
# through any handle, not only through `self` in a method. An assignment
# (`h.values = [7]`), a nested class's field through a field
# (`self.inner.value = 1`), a `ref mut` of a field and `iter_mut` over one all
# take the field's access, checked at run time (VIII.3, `[EXC-16]`,
# `[EXC-18]`); they were refused with "in this phase" (D-462).

class Inner:
    value: i32

class Outer:
    inner: Inner

    fn bump(mut self):
        self.inner.value = 1

class Holder:
    values: Array[i32]

fn main():
    o = Outer(Inner(0))
    o.bump()
    println(o.inner.value)
    h = Holder([1, 2])
    h.values = [7]
    alias = h
    alias.values.push(8)
    r = ref mut h.values
    r.push(9)
    println(h.values)
    for x in h.values.iter_mut():
        x += 1
    println(h.values)
