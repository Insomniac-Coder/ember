#$ test: compile-fail
#$ rules: CTL-7
# `[CTL-7]` — `break` and `continue` in a `defer` inside a loop are `E2160`, and nothing else.

fn main():
    for i in 0..3:
        defer:
            break                   #$ error[E2160]: control flow cannot leave a `defer` block
    while true:
        defer:
            continue                #$ error[E2160]: control flow cannot leave a `defer` block
