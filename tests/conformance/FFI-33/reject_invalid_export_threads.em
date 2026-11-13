#$ test: compile-fail
#$ rules: FFI-33, ATT-6
#$ profiles: debug
#$ error[E0104]: `threads` in `@export` must be `main`, `any`, or `creator`
#$ error[E0104]: duplicate `threads` in `@export`
#$ error[E0104]: duplicate `on_panic` in `@export`

# SAFETY: nothing else in the process defines this name
@unsafe(export(threads=elsewhere))
fn malformed() -> i32:
    return 0

# SAFETY: nothing else in the process defines this name
@unsafe(export(threads=main, threads=any))
fn duplicate_threads() -> i32:
    return 0

# SAFETY: nothing else in the process defines this name
@unsafe(export(on_panic=abort, on_panic=abort))
fn duplicate_panic() -> i32:
    return 0

fn main():
    pass
