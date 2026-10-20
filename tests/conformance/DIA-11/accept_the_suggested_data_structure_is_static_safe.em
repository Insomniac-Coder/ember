#$ test: run-pass
#$ rules: DIA-11, EFF-13
#$ stdout: 3
## `[DIA-11]` — what the loop's suggestion leads to: the counter as a value, changed through a
## `ref mut`, keeps no dynamic check, so the function is `@static_safe`.

struct Counter:
    value: i32

@static_safe
fn run(mut counter: Counter, n: int):
    for i in 0..n:
        counter.value = counter.value + 1

fn main():
    counter = Counter(0)
    run(counter, 3)
    println(counter.value)
