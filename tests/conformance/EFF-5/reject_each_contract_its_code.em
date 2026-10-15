#$ test: compile-fail
#$ rules: EFF-5, EFF-20, EFF-18, DET-1
#$ error[E4001]: `@noalloc` function `grows` reaches an allocation
#$ error[E4041]: `@noio` function `prints` reaches I/O
#$ error[E4003]: `@noblock` function `waits` reaches an operation that may wait
#$ error[E4040]: `@nopanic(explicit)` function `stops` reaches a panic
#$ error[E4070]: `@deterministic` function `wobbles` reaches a nondeterministic operation
#$ error[E4002]: `@nosync` function `shares` reaches a synchronising operation
# `[EFF-5]` — each contract forbids its effect, and a violation is the contract's code: `E4001`
# for an allocation, `E4041` for console output (`[EFF-20]`), `E4003` for `input`, which may wait
# (and is `Io` too: effects are independent, `[EFF-18]`), `E4040` for a written panic, `E4070`
# for the platform's `sin` (`[DET-1]`), and `E4002` for retaining a `@sync` handle.

@sync
class Token:
    n: int

@noalloc
fn grows(mut xs: Array[int]):
    xs.push(1)

@noio
fn prints(n: int):
    println(n)

@noblock
fn waits() -> String:
    return input()

@nopanic(explicit)
fn stops(n: int) -> int:
    if n < 0:
        panic("negative")
    return n

@deterministic
fn wobbles(x: f64) -> f64:
    return x.sin()

@nosync
fn shares(t: Token) -> int:
    copies = [t, t]
    return copies.len()

fn main():
    xs: Array[int] = []
    grows(xs)
    prints(1)
    println(waits(), stops(1), wobbles(1.0), shares(Token(1)))
