#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, TST-14
#$ profiles: debug, development, release
#$ warning[L3001]: potential reference cycle

# The graph must inspect `Holder[Child]` after substituting `T`, not treat the
# generic declaration as opaque. Its `value` field, an owner of a `Child`, closes this cycle.
[T]
class Holder:
    value: T

class Parent:
    holder: Holder[shares mut Child]

class Child:
    parent: shares mut Parent

fn main():
    return
