#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, TST-14
#$ profiles: debug, release, shipping
#$ warning[L3001]: potential reference cycle

# The graph must inspect `Holder[Child]` after substituting `T`, not treat the
# generic declaration as opaque. Its `value: Child` field closes this cycle.
class Holder[T]:
    value: T

class Parent:
    holder: Holder[Child]

class Child:
    parent: Parent
