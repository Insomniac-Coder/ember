# A15: scoped tasks that only read a plain (not @sync) class object.
import std.thread

class Config:
    scale: int = 2

fn run(cfg: Config, xs: Array[int]) -> int:
    with scope = thread.scope():
        a = scope.spawn(fn() => xs[0] * cfg.scale)
        b = scope.spawn(fn() => xs[1] * cfg.scale)
        return a.join() + b.join()

fn main():
    println(run(Config(), [1, 2]))
