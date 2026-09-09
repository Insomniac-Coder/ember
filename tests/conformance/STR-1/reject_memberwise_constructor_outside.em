#$ test: compile-fail
#$ rules: STR-1, MOD-7
## "It is `pub` iff all fields are `pub` (a `pub(read)` field makes it private
## to the declaring module, since construction is a write; `[MOD-7]`)."

from support.health import Health

fn main():
    h = Health(1, 2)      #$ error[E1020]: `support.health.Health`'s memberwise constructor is private to its module
    println(h.value)
