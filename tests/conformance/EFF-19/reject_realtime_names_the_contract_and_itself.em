#$ test: compile-fail
#$ rules: EFF-19
#$ error[E4001]: `@realtime` (`@noalloc`) function `frame` reaches an allocation
#$ error[E4041]: `@noio` function `log` reaches I/O
# `[EFF-19]` — `@realtime` is, by default, `@noalloc @nolock @noblock @nopanic(explicit)`: a
# diagnostic arising from it names both the failed contract and `@realtime`. An allocation's failure
# is reported once, as the allocation. It is not `@noio`, which `log` has on its own.

@realtime
fn frame(n: int) -> int:
    xs = [n]
    return xs.len()

@noio
fn log(n: int):
    println(n)

fn main():
    log(frame(1))
