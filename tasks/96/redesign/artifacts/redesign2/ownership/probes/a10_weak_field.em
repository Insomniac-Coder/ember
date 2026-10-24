# A10: a back pointer written as a field modifier instead of Weak[Node].
class Node:
    name: String
    weak parent: Option[Node] = None
    children: Array[Node] = []
