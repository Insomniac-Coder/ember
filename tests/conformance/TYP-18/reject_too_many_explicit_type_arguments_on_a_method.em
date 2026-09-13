#$ test: compile-fail
#$ rules: TYP-18, ARN-1

fn main():
    arena = Arena.with_capacity(16)
    stored = arena.alloc[i32, i64](41) #$ error[E2020]: `Arena.alloc` takes one type argument, found 2
