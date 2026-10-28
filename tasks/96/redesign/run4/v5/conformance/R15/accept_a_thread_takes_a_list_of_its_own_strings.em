#$ test: run-pass
#$ rules: THR-1
#$ stdout: 1
## design rules: R15
## R15: the line E7015 gives: the list holds strings of its own, so `fn() owns words` hands it to the
## thread whole (completeness G21's nearest allowed).

import std.thread

fn main():
    text = String(" a ")
    words: Array[String] = [String(text.trim())]
    t = thread.spawn(fn() owns words: println(words.len()))
    t.join()
