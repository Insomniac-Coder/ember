#$ test: compile-pass
#$ rules: WK-5, WK-6, WK-7, TST-14
#$ profiles: debug, release, shipping
#$ warning[L3001]: potential reference cycle

# A self edge is a one-node strong SCC and must receive the same diagnostic as
# the multi-class cycle shapes.
class Node:
    next: Node
