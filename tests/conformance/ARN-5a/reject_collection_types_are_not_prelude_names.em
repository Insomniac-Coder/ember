#$ test: compile-fail
#$ rules: ARN-5a

fn main():
    value: ArenaArray[i32]  #$ error[E1010]: cannot find type `ArenaArray` in this scope
