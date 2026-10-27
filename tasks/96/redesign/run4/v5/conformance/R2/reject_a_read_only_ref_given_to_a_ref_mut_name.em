#$ test: compile-fail
#$ rules: R2
#$ help: to point `r` at `b` write `r = ref mut b`, or for a read-only name `rb = ref b`
## R2: the line that gives a `ref mut` name another ref says so with `ref mut` again, for every type;
## `r = ref b` is refused (E3023; AN-28's message).

fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    r = ref b                  #$ error[E3023]: `r` may change what it refers to, and `ref b` only reads `b`
    println(a, b)
