# `ember inspect --cycle` consumes this complete source graph. The `root`
# weak edge is reported but cannot contribute to the all-strong cycle.
class Root:
    child: Child

class Child:
    root: Weak[Root]
    leaf: Shared[Leaf]

class Leaf:
    root: Root
