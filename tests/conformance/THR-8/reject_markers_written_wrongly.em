#$ test: compile-fail
#$ rules: TYP-43, THR-8, THR-1
# ODR-111 — `Send` and `Sync` are written by hand only with `unsafe extend`, for a struct or enum of
# this package, with nothing else in the block; a class is either only as `@sync`; `Copy` comes from
# `@derive(Copy)`, and `Sized` is never written.

class Node:
    v: int

struct Raw:
    p: *mut int

struct Plain:
    x: int

extend Raw implements Send:    #$ error[E3100]: implementing `Send` by hand requires `unsafe extend`: the compiler cannot check it
    pass

# SAFETY: every change to it is synchronised
unsafe extend Node implements Sync:    #$ error[E2040]: a class is `Sync` only as `@sync`: its reference counts must be atomic
    pass

# SAFETY: it keeps the promises the interface states
unsafe extend Plain implements Copy:    #$ error[E2040]: `Copy` is not written in an `extend`: declare the type `@derive(Copy)`
    pass

# SAFETY: it keeps the promises the interface states
unsafe extend Plain implements Sized:    #$ error[E2040]: `Sized` is never written: every type but `dyn I` and an unsized foreign type is `Sized`
    pass

fn main():
    pass
