#$ test: run-pass
#$ rules: THR-1, CLO-1
#$ stdout: 3
## design rules: R15, R12
## R15 (verdicts 65, 93 and 94): the line E3026 gives: each `thread.spawn` thread takes its own copy of
## the loop's text, `copy of t` (one allocation and copy per thread), so it may outlive the loop; the
## function holds only its copy, so any thread may run it (`thread_shared fn()`).

import std.thread

fn main():
    texts = [String("a b"), String("c")]
    handles: Array[thread.JoinHandle[int]] = []
    for t in texts:
        handles.push(thread.spawn(fn() copy of t -> int: t.split_whitespace().count()))
    total = 0
    for h in owns handles:
        total += h.join()
    println(total)
