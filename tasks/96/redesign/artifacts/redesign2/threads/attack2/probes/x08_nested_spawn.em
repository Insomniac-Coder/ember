# A task that spawns into its own scope (the scope rule must see the nested task's effects).
import std.thread

fn main():
    xs = [1, 2, 3]
    with scope = thread.scope():
        scope.spawn(fn(): scope.spawn(fn(): println(xs[0])))
        println(xs[1])
