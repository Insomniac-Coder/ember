# Clause THR-17.2: "**The loop's own variable** (...) and a local of one round (...) are given to a
# task or a thread with a word"; "`ref t` where the function may still run after the element changes
# or ends is `E3026`"; "The loop's name used with no word is `E3026` for a scoped task, whose message
# gives both words (...), and `E7004` for a `thread.spawn` thread, whose message gives `copy of t`."
#$ test: compile-fail
#$ rules: THR-17, THR-11, CLO-4
# The spec's two whole messages; the `E7004` claims the one part the spec gives, `copy of t`.

import std.thread

fn count_words(texts: Array[String]):
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(owns fn(): println(t.len()))    #$ error[E3026]: the task may still run in a later round, when `t` names another text; give it its own copy, `scope.spawn(owns fn() copy of t: ...)`, or let it look at this round's text, `scope.spawn(owns fn() ref t: ...)`
    for t in texts:
        thread.spawn(owns fn() ref t: println(t.len()))    #$ error[E3026]: the thread may still run after the loop has moved on and `texts` is gone; give it its own copy: `thread.spawn(owns fn() copy of t: ...)`
    for t in texts:
        thread.spawn(owns fn(): println(t.len()))    #$ error[E7004]: `copy of t`
    println(total)

fn main():
    count_words([String("a b")])
