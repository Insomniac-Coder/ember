#$ test: run-pass
#$ rules: WK-5, WK-6, WK-8

# One run creates two disjoint three-object SCCs; the debug report must list
# each component separately instead of merging them into one ownership graph.
class Node:
    next: Option[shares mut Node] = None    #$ warning[L3001]: potential reference cycle

    fn link(owns other: shares mut Node):
        this.next = Some(other)

fn make_cycle():
    first = Node()
    second = Node()
    third = Node()
    first.link(shares mut second)
    second.link(shares mut third)
    third.link(shares mut first)

fn main():
    make_cycle()
    make_cycle()
