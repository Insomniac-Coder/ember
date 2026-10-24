# Attack on the scope rule: a task spawned inside a callee that was handed the scope.
# The task reads xs after start_reader returns; the caller then grows xs (a reallocation).
import std.thread

fn start_reader(scope: thread.Scope, xs: Array[int]):
    scope.spawn(fn(): println(xs[0] + xs[1]))

fn main():
    xs = [1, 2, 3]
    with scope = thread.scope():
        start_reader(scope, xs)
        xs.push(4)
