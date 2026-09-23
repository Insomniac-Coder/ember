#$ test: run-pass
#$ rules: CTL-1
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 2
#$ 7
#$ 8
#$ 4
#$ 5
# D-193 — a list literal iterates (it lives until the loop ends), and so do a
# fixed array and a shared view.

fn main():
    for x in [1, 2]:
        println(x)
    fixed: [int; 2] = [7, 8]
    for x in fixed:
        println(x)
    xs = [4, 5]
    view: Span[int] = xs
    for x in view:
        println(x)
