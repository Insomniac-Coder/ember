#$ test: run-pass
#$ rules: DRP-4, DRP-1
#$ stdout: 3
#$ stdout: dropping 2
#$ stdout: dropping 1
## `[DRP-4]` — a `drop` body that returns lets every drop still due run, in
## reverse declaration order; contrast `run_fail_a_panicking_drop_ends_the_process`.

struct Guard:
    id: i32

    fn drop(mut self):
        println(f"dropping {self.id}")

fn main():
    first = Guard(1)
    second = Guard(2)
    println(first.id + second.id)
