# Clause CLS-4.1: "a final class's markers are read from its fields"
#$ test: run-pass
#$ rules: CLS-4, THR-8
#$ stdout: 1
# `Job` writes no promise; its fields make it `Send`, so it meets the bound. Needs M9's markers.

class Job:
    n: int = 3

[T: needs Send]
fn sendable(x: T) -> int:
    return 1

fn main():
    println(sendable(Job()))
