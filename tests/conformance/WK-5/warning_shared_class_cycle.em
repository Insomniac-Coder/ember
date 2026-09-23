#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, HEAP-3, HEAP-6, TST-14, TST-26
#$ profiles: debug, release, shipping
#$ warning[L3001]: potential reference cycle

# Shared class handles are strong owners, unlike Weak handles; they participate
# in the visible graph but keep their declared Shared ownership contract.
class Parent:
    child: Shared[Child]

class Child:
    parent: Shared[Parent]

fn main():
    return
