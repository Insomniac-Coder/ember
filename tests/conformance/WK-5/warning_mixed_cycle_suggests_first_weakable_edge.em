#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, WK-10, HEAP-3, TST-14, TST-26
#$ profiles: debug, release, shipping
#$ warning[L3001]: potential reference cycle

# A Shared edge is not a legal Weak rewrite, but the later direct class edge
# closes the same cycle and can safely become a weak back-reference.
class Owner:
    child: Shared[Child]

class Child:
    parent: Owner

fn main():
    return
