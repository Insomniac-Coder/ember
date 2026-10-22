#$ test: run-pass
#$ rules: HR-25
#$ stdout: 9
#$ assert-c: contains("int64_t em_moved__rl(int64_t x) {")
#$ assert-c: !contains("em_kept__rl")
#$ assert-c: contains("{ "em_kept", NULL, NULL, NULL, 0, UINT64_C(")
# `#! module reloadable` makes the module's functions reloadable under `reload = "opt-in"`; an
# item's own `@noreload` wins over it.

#! module reloadable

fn moved(x: int) -> int:
    return x + 2

@noreload
fn kept(x: int) -> int:
    return x * 3

fn main():
    println(kept(moved(1)))
