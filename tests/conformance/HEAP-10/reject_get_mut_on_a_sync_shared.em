#$ test: compile-fail
#$ rules: HEAP-10, TYP-45
# `[TYP-45]` — a `thread_shared T` reads as its `T`; it has no `get_mut`.

fn main():
    s = thread_shared 5
    s.get_mut()    #$ error[E2072]: a `thread_shared i64` reads as an `i64`; remove `.get_mut()`
