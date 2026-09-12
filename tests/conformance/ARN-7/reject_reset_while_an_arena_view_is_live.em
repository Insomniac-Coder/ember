#$ test: compile-fail
#$ rules: ARN-7, BRW-1
# reset needs exclusive access to the arena and therefore conflicts with a
# returned allocation whose arena borrow remains live.

fn main():
    arena = Arena.with_capacity(64)
    value: ref mut i32 = arena.alloc(1)
    arena.reset()                  #$ error[E3021]: `arena` is borrowed here and mutably borrowed elsewhere
    println(value)
