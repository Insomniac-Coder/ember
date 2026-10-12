#$ test: run-fail
#$ rules: DRP-4, DRP-1, PAN-1
#$ panics: drop failed
#$ stdout: 1
#$ stdout: 2
#$ stdout: dropping 2
## `[DRP-4]` — a `drop` body that panics in `abort` mode ends the process: the
## drops still due (`first`'s) do not run.

struct Guard:
    id: i32

    fn drop(mut self):
        println(f"dropping {self.id}")
        if self.id == 2:
            panic("drop failed")

fn main():
    first = Guard(1)
    second = Guard(2)
    println(first.id)
    println(second.id)
