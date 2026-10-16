#$ test: compile-fail
#$ rules: CORO-10, FN-6
#$ profiles: debug
# `[CORO-10]` — a `gen fn` given to C as a function pointer is `E2222`.

gen fn ticks(n: i32) -> Generator[i32]:
    yield n

fn main():
    callback: extern "C" fn(i32) -> i32 = ticks    #$ error[E2222]: a `gen fn` cannot be passed to C
    println(callback(1))
