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
# through a `ref mut` parameter, `mem.swap` or a `ref mut`. A view copied out of
# an `Array` of `static` views is `static` too.

class Label:
    text: StrView

struct P:
    a: StrView
    b: StrView

fn put(ref mut x: StrView, v: StrView):
    x = v

fn advance(ref mut s: StrView):
    s = s[1..]

[T]
fn store(ref mut xs: Array[T], owns x: T):
    xs.push(owns x)

fn swap_fields(ref mut p: P):
    t = p.a
    p.a = p.b
    p.b = t

fn main():
    names = ["ann", "bob"]
    store(ref mut names, owns "cy")
    old = mem.replace(ref mut names[0], "amy")
    label = Label("lit")
    label.text = names[1]
    label.text = "lit"
    copy = [n for n in names]
    copy.push(owns "dee")
    names.extend(["dee"])
    println(names)
    s = String.from("hello")
    name = "x"
    put(ref mut name, s.as_str())
    println(label.text, name)
    advance(ref mut name)
    other = old
    r = ref mut other
    r = names[0]
    println(name, old)
    a = "first"
    b = "second"
    mem.swap(ref mut a, ref mut b)
    p = P(a, b)
    swap_fields(ref mut p)
    println(p.b, p.a)
    println(copy)
