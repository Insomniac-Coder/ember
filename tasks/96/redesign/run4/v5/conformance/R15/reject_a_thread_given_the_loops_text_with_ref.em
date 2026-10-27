#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdict 93): "`ref t` is refused where the function may still run after the element changes or
## ends: a `thread.spawn` thread may outlive the loop (E3026: "the thread may still run after the loop has
## moved on and `texts` is gone; give it its own copy: `thread.spawn(fn() copy of t: ...)`")" (the lead's
## code and message, worklist section 3, with the line to write). The nearest allowed program is
## `accept_a_thread_takes_a_copy_of_the_loops_text.em`.

import std.thread

fn main():
    texts = [String("a b"), String("c")]
    handles: Array[thread.JoinHandle[int]] = []
    for t in texts:
        handles.push(thread.spawn(fn() ref t -> int: t.split_whitespace().count()))    #$ error[E3026]: the thread may still run after the loop has moved on and `texts` is gone; give it its own copy: write `handles.push(thread.spawn(fn() copy of t -> int: t.split_whitespace().count()))` in place of line 15
    total = 0
    for h in owns handles:
        total += h.join()
    println(total)
