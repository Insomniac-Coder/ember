#$ test: run-pass
#$ rules: WK-5, WK-6, WK-8

# A live class object that retains itself remains allocated after `main` drops
# its only local handle. `ember run --leak-check` must report this SCC at
# runtime; ordinary `run-pass` verification confirms the program itself is
# still valid when the diagnostic mode is disabled.
class Node:
    next: Option[shares mut Node] = None    #$ warning[L3001]: potential reference cycle
    previous: weakref Node = None

    static fn link(owns me: shares mut Node):
        me.next = Some(shares mut me)
        me.previous = weakref me

fn main():
    node = Node()
    Node.link(shares mut node)
