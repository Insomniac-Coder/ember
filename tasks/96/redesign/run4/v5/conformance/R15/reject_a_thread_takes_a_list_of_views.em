#$ test: compile-fail
#$ rules: THR-1, THR-2, THR-10
## design rules: R15, R7
## R15: "A one-thread owner or a lent value reaching another thread is E7015": a list of views of
## `main`'s text is lent (R7), and an unscoped thread may outlive it (completeness G21; the code as
## `V4/changes.md` section 5 reads R15).

import std.thread

fn main():
    text = String(" a ")
    words: Array[StrView] = [text.trim()]
    t = thread.spawn(fn() owns words: println(words.len()))    #$ error[E7015]: `words` holds views of `text` (line 11), which stays on this thread and may end before the thread does, so the thread cannot take them; give the thread text of its own: write `words: Array[String] = [String(text.trim())]` instead of line 11
    t.join()
