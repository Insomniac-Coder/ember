#$ test: compile-fail
#$ rules: BRW-2, BRW-1
# Move the write one line earlier and the borrow is still live, because a use
# of `r` comes after it. Liveness, not scope, is what decides.

fn main():
    x: i32 = 1
    r: ref i32 = ref x
    x = 5                  #$ error[E3021]: `x` cannot be written while it is borrowed
    println(r)
