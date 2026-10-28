#$ test: run-pass
#$ rules: R15
#$ stdout: 1
## R15: the line E7015 gives: the list holds strings of its own, so `with owns words` hands it to the
## thread whole (completeness G21's nearest allowed).

import std.thread

fn main():
    text = String(" a ")
    words: Array[String] = [String(text.trim())]
    t = thread.spawn(fn() with owns words: println(words.len()))
    t.join()
