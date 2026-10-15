#$ test: run-pass
#$ rules: HEAP-4
#$ stdout: value 5
# D-595 — a function with a `str` parameter (passed as two words on Windows, D-569) that reads a
# `Shared` through `get`: the access check named the function's C symbol in its message, and the
# backend stopped at it.

fn show(s: Shared[int], label: str):
    println(label, s.get())

fn main():
    show(Shared(5), "value")
