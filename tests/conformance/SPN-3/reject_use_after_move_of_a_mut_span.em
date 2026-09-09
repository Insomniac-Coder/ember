#$ test: compile-fail
#$ rules: SPN-3, OWN-3
# "`MutSpan[T]` is move-only" — `m2 = m` moves, so `m.len()` afterwards is
# `E3040`. (Passing `m` to a `mut` parameter instead would implicitly reborrow
# per `[FN-1]`/D5, which is why the violation here is a plain rebinding.)

fn main():
    buf: Array[i32] = Array[i32]()
    buf.push(1)
    m = buf.as_mut_span()
    m2 = m
    println(m2.len())
    println(m.len())   #$ error[E3040]: `m` has been moved out of
