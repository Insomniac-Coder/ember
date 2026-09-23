#$ test: compile-fail
#$ rules: ATT-1
#$ profiles: debug
#$ error[E0104]: `@gpu` is reserved for a later version
# `@nopanic` (without `(explicit)`), `@no_runtime_checks`, `@allocator` and
# `@gpu` are reserved, and rejected naming that.

@gpu
fn kernel():
    pass

fn main():
    kernel()
