#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdicts 65 and 93): "The loop's name used with no word is refused: E3026 for a scoped task,
## whose message gives both words" (`copy of t`, its own copy, or `ref t`, a look at this round's
## element; section 3's E3026 message, with the line to write). The nearest allowed programs are
## `run_scoped_tasks_add_their_parts_to_a_total.em` and
## `accept_a_scoped_task_looks_at_the_loops_text_with_ref.em`.

import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())    #$ error[E3026]: the task may still run in a later round, when `t` (line 14) names another text; give it its own copy, `scope.spawn(fn() copy of t: total += t.split_whitespace().count())`, or let it look at this round's text, `scope.spawn(fn() ref t: total += t.split_whitespace().count())`, in place of line 15
    return total

fn main():
    println(count_words([String("a b"), String("c")]))
