#$ test: run-pass
#$ rules: UNS-5
#$ stdout: 0 2 1 0 1
#$ stdout: 5
# `[UNS-5]` — `zeroed`, `swap`, `replace`, `take`, `drop` and `keep_alive` are safe: each is called
# here outside unsafe code.

import std.mem
from std.mem import zeroed

fn main():
    z: i32 = zeroed[i32]()
    a = 1
    b = 2
    mem.swap(ref mut a, ref mut b)
    old = mem.replace(ref mut b, 0)
    taken = mem.take(ref mut a)
    println(z, taken, old, b, a + 1)
    s = String("12345")
    mem.keep_alive(s)
    println(s.len())
    mem.drop(s)
