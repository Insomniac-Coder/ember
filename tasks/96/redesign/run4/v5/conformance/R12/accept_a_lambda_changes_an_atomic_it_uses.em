#$ test: run-pass
#$ rules: CLO-1, FN-4
#$ stdout: 2
## design rules: R12, R5
## Verdict 86 (verdict 75's spelling is void), R12 (rules.md lines 662-672, 752-756): the lambda changes
## `main`'s real `hits`, an `Atomic`, through its own operation (R5, lines 405-410); nothing is written on
## the lambda or on `main`.

from std.sync import Atomic

fn main():
    hits = Atomic(0)
    count_hit = fn(): hits.fetch_add(1)
    count_hit()
    count_hit()
    println(hits.load())
