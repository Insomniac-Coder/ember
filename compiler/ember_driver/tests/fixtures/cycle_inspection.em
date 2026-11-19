# `ember inspect --cycle` consumes this complete source graph. The `root`
# weak edge is reported but cannot contribute to the all-strong cycle.
class Root:
    child: Child

class Child:
    root: weakref Root
    leaf: shares Leaf

class Leaf:
    root: Root
