#$ test: compile-fail
#$ rules: STR-1, MOD-7
## "It is `pub` iff all fields are `pub` (a `pub(read)` field makes it private
## to the declaring module, since construction is a write; `[MOD-7]`)."

from support.health import Health

fn main():
    h = Health(1, 2)      #$ error[E1052]: `Health(...)` is not visible here: `value` is `pub(read)`, and construction is a write, so a `Health` is built only inside its module (`support.health`)
    println(h.value)
