#$ test: compile-fail
#$ rules: ARN-6, ARN-7
# The scope holds a mutable borrow of the parent for its whole region. Parent
# allocation is rejected with the arena-specific A1 diagnostic.

fn main():
    arena = Arena.with_capacity(32)
    scope = arena.scope()
    parent: ref mut i32 = arena.alloc(1)  #$ error[E3096]: `arena` is scoped here
    child: ref mut i32 = scope.alloc(2)
    println(parent + child)
