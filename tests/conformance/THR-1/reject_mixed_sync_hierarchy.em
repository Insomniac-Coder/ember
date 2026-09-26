#$ test: compile-fail
#$ rules: THR-1
#$ profiles: debug
#$ error[E7001]: class `Child` and its base `Parent` must both be `@sync` or both be non-`@sync`

open class Parent:
    value: int = 1

@sync
class Child(Parent):
    more: int = 2
