#$ test: compile-fail
#$ rules: OWN-9
#$ help: remove `owns`: write `b = a`; for a copy write `b = copy of a`; for a second owner write `b = shares a`
## design rules: R3
## R3: `owns x` before a value is refused where the value is already handed over by its place
## (`b = a`) (E0109; AN-47's message).

fn main():
    a = [1, 2]
    b = owns a                 #$ error[E0109]: `owns a` is not needed: `b = a` already hands `a` over
    println(b.len())
