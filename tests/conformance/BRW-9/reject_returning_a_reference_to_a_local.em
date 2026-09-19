#$ test: compile-fail
#$ rules: BRW-9, LT-1, LT-5, DIA-7
#$ profiles: debug, release, shipping
#$ error[E3060]: does not live long enough

# A `ref` is safe only while its source place is alive. Returning this borrow
# would expose a pointer into `local` after the callee frame has ended.
fn dangling() -> ref i32:
    local: i32 = 42
    return ref local

fn main():
    pass
