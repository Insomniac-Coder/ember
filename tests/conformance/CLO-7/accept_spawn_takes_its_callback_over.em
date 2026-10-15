#$ test: run-pass
#$ rules: CLO-7, THR-10, CLO-15
#$ stdout: 19
# `[CLO-7]` — `thread.spawn` keeps its callback beyond the caller's borrows, so it takes it over:
# a lambda written at the call site captures by move (`owned fn`): the `String` moves to the thread,
# which drops it.

import std.thread

fn main():
    message = "hello from a thread".to_string()
    worker = thread.spawn(owned fn() => message.len())
    println(worker.join())
