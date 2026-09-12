#$ test: run-pass
#$ rules: ARN-1, ARN-6, ARN-7, LT-4
# Each scope holds its parent's mutable borrow, allocates from the same stable
# chunk chain, and rewinds to its own mark on block exit. Nested scopes repeat
# the same mechanism rather than introducing a fixed nesting limit.

fn main():
    arena = Arena.with_capacity(32)
    before: ref mut i32 = arena.alloc(1)
    println(before)
    with scope = arena.scope():
        outer: ref mut i32 = scope.alloc(2)
        println(outer)
        with nested = scope.scope():
            inner: ref mut i32 = nested.alloc(3)
            println(inner)
        after_nested: ref mut i32 = scope.alloc(4)
        println(after_nested)
    after_scope: ref mut i32 = arena.alloc(5)
    println(after_scope)
#$ stdout: 1
#$ 2
#$ 3
#$ 4
#$ 5
#$ assert-c: contains("ember_arena_mark")
#$ assert-c: contains("ember_arena_rewind")
