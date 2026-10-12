#$ test: run-fail
#$ rules: PAN-1, PAN-3, DRP-1
#$ panics: stop here
#$ stdout: 1
## `[PAN-1]` — there is no unwinding: a panic aborts, and no destructor or
## `defer` block runs after it. So `[PAN-3]`'s case, a `drop` panicking while
## the process is already panicking, cannot arise: no `drop` runs then at all.

struct Guard:
    id: i32

    fn drop(mut self):
        println("dropping")
        panic("a second panic")

fn main():
    guard = Guard(1)
    defer:
        println("deferred")
    println(guard.id)
    panic("stop here")
