#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, WK-5, WK-8

# A Shared payload can keep its own control block alive. It is a runtime cycle
# even though the static class-ownership lint has no class declaration to
# predict, so the report must identify it without claiming L3001 proved it.
struct Node:
    next: Option[Shared[Node]]

fn main():
    node = Shared(Node(None))
    alias = node
    payload = node.get_mut()
    payload.next = Some(alias)
