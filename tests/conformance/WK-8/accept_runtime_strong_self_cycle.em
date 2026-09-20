#$ test: run-pass
#$ rules: WK-5, WK-6, WK-8

# A live class object that retains itself remains allocated after `main` drops
# its only local handle. `ember run --leak-check` must report this SCC at
# runtime; ordinary `run-pass` verification confirms the program itself is
# still valid when the diagnostic mode is disabled.
class Node:
    next: Option[Node] = None
    previous: Weak[Node] = Weak[Node].empty()

    fn link(mut self):
        self.next = Some(self)
        self.previous = Weak(self)

fn main():
    node = Node()
    node.link()
