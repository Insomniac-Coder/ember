#$ test: compile-fail
#$ rules: TYP-15, LT-3, LT-4, LT-4a
# Arena is a narrow non-view provenance source, not static storage. Its result
# must remain bounded by the Arena even after passing through a wrapper.

@borrows(arena)
fn allocate(arena: Arena) -> ref mut i32:
    return arena.alloc(9)

fn main():
    arena = Arena.with_capacity(16)
    slot = allocate(arena)
    _boxed: Box[ref mut i32] = Box[ref mut i32](slot) #$ error[E3063]: `ref mut i32` is a view, so it may not be stored in a Box's contents
