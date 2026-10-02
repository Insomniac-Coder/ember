#$ test: compile-fail
#$ rules: ATT-6, CLI-19
#$ profiles: debug
#$ error[E0900]: `@must_use` is not implemented yet
#$ error[E0900]: only `on_panic=abort` is implemented for exported functions
#$ error[E0900]: `@unroll` is not implemented yet
#$ error[E0900]: deriving `Ord` is not implemented yet
# A listed attribute whose effect is not built is rejected with `E0900`, never
# accepted and ignored.

@must_use
fn twice(x: int) -> int:
    return x * 2

@export(on_panic=unwind)
extern "C" fn exported(x: i32) -> i32:
    return x

@derive(Copy, Ord)
struct Key:
    id: int

fn main():
    total = 0
    @unroll(2)
    for i in 0..2:
        total += twice(i)
    println(total)
