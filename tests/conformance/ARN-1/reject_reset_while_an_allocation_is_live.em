#$ test: compile-fail
#$ rules: ARN-1, ARN-7, BRW-1
# reset takes a mutable borrow of the arena. The shared arena borrow carried by
# the returned view remains live through its last use, so reset conflicts.

fn main():
    arena = Arena.with_capacity(64)
    value: ref mut i32 = arena.alloc(1)
    arena.reset()                  #$ error[E3021]: `arena` is borrowed here and mutably borrowed elsewhere
    println(value)
