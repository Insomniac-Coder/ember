#$ test: compile-fail
#$ rules: LEX-15
# ODR-035 — an item starting `union` and a name is where a `union`
# declaration would go, and that stays reserved for a later version.

union Bits:  #$ error[E0005]: `union` is reserved for a later version
    x: int

fn main():
    pass
