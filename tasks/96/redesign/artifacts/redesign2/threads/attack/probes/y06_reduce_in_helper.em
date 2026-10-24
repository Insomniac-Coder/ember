# PROPOSED syntax. A reduction written inside a function the task calls (6.3 rule 4 / 6.5).
# add_words is compiled once; called from main it must add to WORDS, called from a task to that task's
# accumulator. There is no accumulator it can name.
import std.thread

WORDS = 0

fn add_words(t: String):
    global WORDS
    WORDS += t.split_whitespace().count()

fn count(texts: Array[String]):
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): add_words(t))
    add_words("one more".to_string())
