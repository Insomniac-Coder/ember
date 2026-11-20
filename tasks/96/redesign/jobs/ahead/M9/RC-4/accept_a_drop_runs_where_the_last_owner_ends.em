# Clause RC-4.3: "the one after a `join` or a scope's end makes a value a joined task ended on its
# own thread end before the parent's next line"; `[THR-20]`: "**A `drop` runs where the last owner
# ends**"; "a `drop` that is not plain never waits in a queue, so it runs where its last owner ends"
#$ test: run-pass
#$ rules: RC-4, THR-20, THR-1
#$ stdout: 1
#$ stdout: ticket ends
#$ stdout: joined
# The thread takes the one owner over and ends it: the `drop`, which prints and so is not plain,
# runs on that thread, before `join` returns.

import std.thread

class Ticket:
    n: int = 1

    fn drop():
        println("ticket ends")

fn main():
    t = thread_shared Ticket()
    h = thread.spawn(owns fn() owns t: println(t.n))
    h.join()
    println("joined")
