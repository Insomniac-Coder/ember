#$ test: compile-fail
#$ rules: SPN-1, BRW-1, STD-2
# Writing the conversion out does not opt out of the borrow. `as_span()` and
# the implicit coercion are the same construction — one producer, one loan —
# so this is rejected for the same reason `v: Span[i32] = a` is.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    v = a.as_span()
    a.push(2)              #$ error[E3021]: `a` is borrowed here and mutably borrowed elsewhere
    println(v[0])
