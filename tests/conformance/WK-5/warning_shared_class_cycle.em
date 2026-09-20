#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, HEAP-3, HEAP-6, TST-14, TST-26
#$ profiles: debug, release, shipping
#$ warning[L3001]: potential reference cycle

# A `Shared[Child]` is a strong owner, unlike `Weak[Child]`; it therefore
# participates in the same visible class ownership graph.
class Parent:
    child: Shared[Child]

class Child:
    parent: Parent

fn main():
    return
