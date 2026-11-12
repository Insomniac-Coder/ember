#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, HEAP-3, HEAP-6, TST-14, TST-26
#$ profiles: debug, development, release
#$ warning[L3001]: potential reference cycle

# `shares` class handles are strong owners, unlike `weakref` handles; they participate
# in the visible graph but keep their declared `shares` ownership contract.
class Parent:
    child: shares mut Child

class Child:
    parent: shares mut Parent

fn main():
    return
