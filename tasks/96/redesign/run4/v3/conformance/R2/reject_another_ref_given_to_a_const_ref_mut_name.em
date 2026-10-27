#$ test: compile-fail
#$ rules: R2
#$ help: to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain `ref mut`: `r = ref mut a`
## R2: a name written `const ref mut` is never given another ref (E3023, verdict 42); the message is
## `V3/rules.md` section 3's.

fn main():
    a = 1
    b = 2
    r = const ref mut a
    r = ref mut b              #$ error[E3023]: `r` is `const ref mut a`, so it always refers to `a`
    r = 30
    println(a, b)
