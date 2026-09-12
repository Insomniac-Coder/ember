#$ test: compile-fail
#$ rules: ARN-1, BRW-7
# Arena owns its chunk chain and is always move-only. Moving it transfers that
# ownership; copying it would make two destructors free the same chunks.

fn main():
    first = Arena.with_capacity(16)
    second = first
    value: ref mut i32 = second.alloc(1)
    println(value)
    first.reset()             #$ error[E3050]: `first` is borrowed after it has been moved out of
