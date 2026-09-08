#$ test: compile-fail
#$ rules: CTL-7
#$ error[E2160]: control flow cannot leave a `defer` block
#$ error[E1010]: `break` outside a loop
#$ error[E1010]: no loop named `missing` is open here

fn leaves() -> i32:
    defer:
        return 1
    return 0

fn stray():
    break

fn wrong_label():
    for i in 0..3:
        break missing
