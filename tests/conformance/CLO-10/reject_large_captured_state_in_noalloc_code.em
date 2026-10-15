#$ test: compile-fail
#$ rules: CLO-10
#$ error[E4001]: a callable value whose captured state is larger than three words
# `[CLO-10]` — captured state larger than three words is placed in one heap allocation, which
# carries `Alloc`: four `int`s are four words.

@noalloc
fn four(a: int, b: int, c: int, d: int) -> fn() -> int:
    return owned fn() => a + b + c + d

fn main():
    f = four(1, 2, 3, 4)
    println(f())
