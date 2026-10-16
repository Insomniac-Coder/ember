#$ test: run-pass
#$ rules: DIA-22, FN-5
#$ profiles: debug
#$ stdout: 6 12 8
# `[DIA-22]` — a call that matches its callee's signature, positionally, by name or leaving a
# default out, causes no diagnostic.

fn area(w: int, h: int = 4) -> int:
    return w * h

fn main():
    println(area(2, 3), area(h=4, w=3), area(2))
