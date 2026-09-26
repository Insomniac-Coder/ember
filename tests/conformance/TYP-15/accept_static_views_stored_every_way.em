#$ test: run-pass
#$ rules: TYP-15, LT-3, LT-21, FN-1
#$ stdout: ['amy', 'bob', 'cy', 'dee']
#$ stdout: lit hello
#$ stdout: ello ann
#$ stdout: second first
#$ stdout: ['amy', 'bob', 'cy', 'dee']
# `[TYP-15]` (D-352, ODR-069) — every way of storing a view is allowed when
# the view outlives where it goes: a `static` one anywhere, including through
# a generic function that pushes its argument (its callers supply `static`
# views), and a local's view in another local that it outlives, including
# through a `mut` parameter, `mem.swap` or a `ref mut`. A view copied out of
# an `Array` of `static` views is `static` too.

class Label:
    text: str

struct P:
    a: str
    b: str

fn put(mut x: str, v: str):
    x = v

fn advance(mut s: str):
    s = s[1..]

fn store[T](mut xs: Array[T], owned x: T):
    xs.push(x)

fn swap_fields(mut p: P):
    t = p.a
    p.a = p.b
    p.b = t

fn main():
    names = ["ann", "bob"]
    store(names, "cy")
    old = mem.replace(names[0], "amy")
    label = Label("lit")
    label.text = names[1]
    label.text = "lit"
    copy = [n for n in names]
    copy.push("dee")
    names.extend(["dee"])
    println(names)
    s = String.from("hello")
    name = "x"
    put(name, s.as_str())
    println(label.text, name)
    advance(name)
    other = old
    r = ref mut other
    r = names[0]
    println(name, old)
    a = "first"
    b = "second"
    mem.swap(a, b)
    p = P(a, b)
    swap_fields(p)
    println(p.b, p.a)
    println(copy)
