#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, WK-5, WK-8

# A `shares` payload can keep its own control block alive. It is a runtime cycle
# even though the static class-ownership lint has no class declaration to
# predict, so the report must identify it without claiming L3001 proved it.
struct Node:
    next: Option[shares mut Node]

fn main():
    node = shares mut Node(None)
    node.next = Some(shares mut node)
