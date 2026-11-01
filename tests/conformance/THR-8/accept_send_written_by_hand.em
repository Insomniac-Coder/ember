#$ test: run-pass
#$ rules: TYP-43, THR-8, TYP-17
#$ profiles: debug, development
#$ stdout: 7
#$ 1 3 4
# ODR-111 — a raw pointer is not `Send`, so neither is a struct holding one; the program may say it
# is with `unsafe extend … implements Send`, taking the responsibility, and a generic one holds for
# the instances whose arguments meet its parameters' marker bounds. The markers are interfaces of
# `std.core`, so a bound names them as any interface (Part IV §8).

import std.thread

struct Raw:
    p: *mut int
    n: int

unsafe extend Raw implements Send:  # SAFETY: the pointer is never read; only `n` is
    pass

[T]
struct Pair:
    a: T
    p: *mut int

unsafe extend[T: needs Send] Pair[T] implements Send:  # SAFETY: as `Raw`
    pass

[T: needs Send]
fn need_send(x: T) -> T:
    return x

[T: needs Sync]
fn need_sync(x: T) -> T:
    return x

[T: needs Copy]
fn need_copy(x: T) -> T:
    return x

fn main():
    r = Raw(null[*mut int](), 7)
    worker = thread.spawn(owns owned fn() => need_send(r).n)
    println(worker.join())
    q = Pair(1, null[*mut int]())
    println(need_send(q).a, need_sync(3), need_copy(4))
