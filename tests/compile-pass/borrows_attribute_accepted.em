#$ test: compile-pass
#$ rules: LT-1a

## The form the rule is written for: two view-typed parameters, and the
## attribute saying which one the result points into, so the caller may keep
## using the other.

@borrows(a)
fn pick(a: ref i32, b: ref i32) -> ref i32:
    return a

fn main():
    x: i32 = 1
    y: i32 = 2
    p: ref i32 = ref x
    q: ref i32 = ref y
    v: i32 = pick(p, q)
    println(v)
