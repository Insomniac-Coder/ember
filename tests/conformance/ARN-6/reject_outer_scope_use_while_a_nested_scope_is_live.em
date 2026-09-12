#$ test: compile-fail
#$ rules: ARN-6, ARN-7
# The same held-parent rule applies recursively to ScopedArena.scope.

fn main():
    arena = Arena.with_capacity(32)
    outer = arena.scope()
    nested = outer.scope()
    earlier: ref mut i32 = outer.alloc(1)  #$ error[E3096]: `outer` is scoped here
    inner: ref mut i32 = nested.alloc(2)
    println(earlier + inner)
