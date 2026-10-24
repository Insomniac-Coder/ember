# A15 attack: the parent grows a list while a scoped task reads it (must stay refused).
import std.thread

fn main():
    xs: Array[int] = [1, 2, 3]
    with s = thread.scope():
        s.spawn(fn() => println(xs[0] + xs[1]))
        xs.push(4)
