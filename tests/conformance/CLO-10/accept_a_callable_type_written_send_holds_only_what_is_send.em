#$ test: run-pass
#$ rules: CLO-10, THR-8, THR-10, FN-6
#$ profiles: debug, development, release
#$ stdout: 4 15 13 30 7 21
# ODR-264 — a written callable type says its values are `Send` with `+ Send`: `fn(A) -> R + Send`.
# A function is one, and so is a lambda whose captures are all `Send`, checked where it is made; a
# struct holding one in a field moves to a thread; a parameter written `f: fn() -> int + Send` takes
# only what is `Send`, as `F: fn() -> int + Send` does; and the value is a `fn(A) -> R` too, the
# same bits. After a return type, `+ Send` is the callable's whose return type it ends:
# `fn() -> fn() -> int + Send` is a `Send` callable giving a `fn() -> int`.

import std.thread

struct Job:
    work: fn(int) -> int + Send

fn twice(x: int) -> int:
    return 2 * x

fn twenty_one() -> int:
    return 21

fn pick() -> fn() -> int:
    return twenty_one

fn make(step: int) -> fn(int) -> int + Send:
    return owned fn(x: int) -> int => x + step

fn later(owned f: fn() -> int + Send) -> int:
    h = thread.spawn(owned fn() -> int => f())
    return h.join()

fn on_a_thread(owned job: Job, x: int) -> int:
    h = thread.spawn(owned fn() -> int => job.work(x))
    return h.join()

fn main():
    label = "abc".to_string()
    a = on_a_thread(Job(twice), 2)
    b = on_a_thread(Job(make(10)), 5)
    c = later(owned fn() -> int => label.len() + 10)
    plain: fn(int) -> int = make(20)
    d = plain(10)
    e = later(owned fn() -> int => 7)
    maker: fn() -> fn() -> int + Send = pick
    task = owned fn() -> int:
        inner = maker()
        return inner()
    h = thread.spawn(task)
    println(a, b, c, d, e, h.join())
