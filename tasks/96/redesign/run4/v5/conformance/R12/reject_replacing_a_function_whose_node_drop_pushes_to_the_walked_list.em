#$ test: compile-fail
#$ rules: R12, R8, R5
#$ help: replace it after the loop: move line 24 below the loop
## R12 ("replacing or ending a function value ends what its state owns ... and that node's `drop`
## counts as called there (inside a walk of what it changes, E3020)"), R8 (c), R5: `Node`'s `drop`
## adds to `LOG`, and replacing `h.keep` inside a loop over `LOG` ends the node `h.keep` owned.

static mut LOG: Array[String] = []

class Node:
    fn drop():
        LOG.push(String("node ended"))

class Hook:
    keep: fn() = fn(): pass

fn main():
    n = Node()
    h = Hook()
    h.keep = fn() owns n: pass
    LOG.push(String("start"))
    for line in LOG:
        println(line)
        h.keep = fn(): pass    #$ error[E3020]: `h.keep = fn(): pass` (line 24) ends the function stored there before and the `Node` it owns, whose `drop` adds to `LOG` (its line 2), while this loop walks `LOG`; replace it after the loop: move line 24 below the loop
