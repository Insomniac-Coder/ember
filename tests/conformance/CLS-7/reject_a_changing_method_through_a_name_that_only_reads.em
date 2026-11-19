#$ test: compile-fail
#$ rules: CLS-7, FN-9, EXC-15
# `[CLS-7]` — a method whose body changes `this` may change it, and is not called through a name that
# only reads it: a plain or `ref` parameter, a `ref` local, a `virtual` method an override of which
# changes its object.

class Node:
    v: int = 0

    fn set():
        this.v = 1

open class Base:
    v: int = 0

    virtual fn go():
        pass

class Derived(Base):
    override fn go():
        this.v = 1

fn plain(n: Node):
    n.set()    #$ error[E3023]: `n.set()` changes the caller's node, but `n` only reads it; write `fn plain(ref mut n: Node):` and call it `plain(ref mut n)`

fn viewed(ref n: Node):
    n.set()    #$ error[E3023]: write `fn viewed(ref mut n: Node):`

fn through_a_base(b: Base):
    b.go()     #$ error[E3023]

fn main():
    n = Node()
    plain(n)
    viewed(n)
    m = ref n
    m.set()    #$ error[E3023]: to change `n` through `m`, write `m = ref mut n`
    through_a_base(Derived())
