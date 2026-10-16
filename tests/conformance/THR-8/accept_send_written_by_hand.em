#$ test: run-pass
#$ rules: THR-8, TYP-17
#$ profiles: debug, release
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

struct Pair[T]:
    a: T
    p: *mut int

unsafe extend[T: Send] Pair[T] implements Send:  # SAFETY: as `Raw`
    pass

fn need_send[T: Send](x: T) -> T:
    return x

fn need_sync[T: Sync](x: T) -> T:
    return x

fn need_copy[T: Copy](x: T) -> T:
    return x

fn main():
    r = Raw(null[*mut int](), 7)
    worker = thread.spawn(owned fn() => need_send(r).n)
    println(worker.join())
    q = Pair(1, null[*mut int]())
    println(need_send(q).a, need_sync(3), need_copy(4))
