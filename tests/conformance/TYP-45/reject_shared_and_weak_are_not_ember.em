#$ test: compile-fail
#$ rules: TYP-45, WK-12, RC-1
# `[TYP-45]` — `Shared` and `Weak` are not types of Ember: naming one is `E1010`, whose message gives
# what to write (`shares T` or `shares mut T` for `Shared[T]`, `weakref T` for `Weak[T]`); a link is
# opened with `exists as` (`[WK-12]`), and an owner reads as its value.

struct Counter:
    value: int

class Node:
    v: int = 1

struct Holder:
    counter: Shared[Counter]          #$ error[E1010]: `Shared` is not a type of Ember: write `shares Counter` or `shares mut Counter`
    link: Weak[Shared[Counter]]       #$ error[E1010]: `Weak` is not a type of Ember: write `weakref Counter`
    node: Weak[Node]                  #$ error[E1010]: `Weak` is not a type of Ember: write `weakref Node`

fn main():
    c = Shared(Counter(value=1))      #$ error[E1010]: `Shared` is not Ember
    w = Weak(c)                       #$ error[E1010]: `Weak` is not Ember
    println(c.get().value)            #$ error[E2072]: a `shares mut Counter` reads as a `Counter`; remove `.get()`
    n = Node()
    e: weakref Node = Weak[Node].empty()  #$ error[E1010]: `Weak.empty()` is not Ember: a link that holds no target is `None`
    match w.upgrade():                #$ error[E1010]: a `weakref` link is opened with `exists as`: `if w exists as s:`
        Some(s):
            println(s.value)
        None:
            println(n.v)
    if e exists as m:
        println(m.v)
