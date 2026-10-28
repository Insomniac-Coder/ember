#$ test: compile-fail
#$ rules: BRW-17, LT-3
#$ help: use `p` before line 34
## design rules: R12, R10
## R12 ("an `owns dyn I` value replaced or ended does the same with what it owns"), R10: replacing
## `slot` ends the `Keeper` it held and the node the keeper owned, so the name `p` opened from a link
## to that node is not used after the line (E3060; step 4's re-check: only the function-value form
## was tested).

interface Holder:
    fn size() -> int

class Node:
    value: int = 0

struct Keeper:
    node: Node

extend Keeper implements Holder:
    fn size() -> int:
        return this.node.value

struct Empty:
    pass

extend Empty implements Holder:
    fn size() -> int:
        return 0

fn main():
    n = Node(value=1)
    w = weakref n
    slot: owns dyn Holder = owns Keeper(node=n)
    if w exists as p:
        slot = owns Empty()
        println(p.value)    #$ error[E3060]: `slot = owns Empty()` (line 34) ends the `Keeper` that `slot` held and the node it owned, which `p` reads; use `p` before line 34
