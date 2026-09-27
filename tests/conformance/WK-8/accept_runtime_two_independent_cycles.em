#$ test: run-pass
#$ rules: WK-5, WK-6, WK-8

# One run creates two disjoint three-object SCCs; the debug report must list
# each component separately instead of merging them into one ownership graph.
class Node:
    next: Option[Node] = None    #$ warning[L3001]: potential reference cycle

    fn link(mut self, other: Node):
        self.next = Some(other)

fn make_cycle():
    first = Node()
    second = Node()
    third = Node()
    first.link(second)
    second.link(third)
    third.link(first)

fn main():
    make_cycle()
    make_cycle()
