#$ test: compile-fail
#$ rules: JOB-1, EFF-5
#$ error[E4001]: `@noalloc` function `large` reaches an allocation
# `[JOB-1]` — a job's state larger than its slot's 64 bytes is one heap allocation (`Alloc`).

import std.jobs

@noalloc
fn large():
    values: [int; 10] = [7; 10]
    jobs.submit(owned fn() => println(values[0]))

fn main():
    large()
