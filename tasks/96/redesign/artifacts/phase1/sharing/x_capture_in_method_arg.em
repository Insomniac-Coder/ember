# Minimal form of the E1010 seen in a15_workaround*.em: a captured name inside a method call's argument.
from std.sync import Atomic

class Box2:
    n: int = 0
    fn add(self, k: int):
        self.n += k

fn main():
    total = Atomic(0)
    n = 5
    f = fn() => total.fetch_add(n)
    f()
    b = Box2()
    g = fn() => b.add(n)
    g()
    println(total.load(), b.n)
