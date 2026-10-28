#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain `ref mut`: `r = ref mut a`
## design rules: R2
## R2: a name written `const ref mut` is never given another ref (E3023, verdict 42); the message is
## `V4/rules.md` section 3's.

fn main():
    a = 1
    b = 2
    r = const ref mut a
    r = ref mut b              #$ error[E3023]: `r` is `const ref mut a`, so it always refers to `a`
    r = 30
    println(a, b)
