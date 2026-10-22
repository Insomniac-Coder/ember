# A singly linked list of integers built from class nodes: push at both ends, pop, reverse, search and print.

class Node:
    value: int
    next: Option[Node] = None

    fn init(self, value: int):
        self.value = value

class LinkedList:
    head: Option[Node] = None
    size: int = 0

    fn push_front(self, value: int):
        node = Node(value)
        node.next = self.head
        self.head = Some(node)
        self.size += 1

    fn pop_front(self) -> Option[int]:
        if Some(node) = self.head:
            self.head = node.next
            self.size -= 1
            return Some(node.value)
        return None

    fn push_back(self, value: int):
        node = Node(value)
        if self.head is None:
            self.head = Some(node)
        else:
            current = self.head.unwrap()
            while Some(n) = current.next:
                current = n
            current.next = Some(node)
        self.size += 1

    fn contains(self, value: int) -> bool:
        current = self.head
        while Some(node) = current:
            if node.value == value:
                return True
            current = node.next
        return False

    fn reverse(self):
        prev: Option[Node] = None
        current = self.head
        while Some(node) = current:
            current = node.next
            node.next = prev
            prev = Some(node)
        self.head = prev

    fn to_array(self) -> Array[int]:
        result: Array[int] = []
        current = self.head
        while Some(node) = current:
            result.push(node.value)
            current = node.next
        return result

    fn show(self):
        parts = [str(v) for v in self.to_array()]
        println(" -> ".join(parts) + f"  (size {self.size})")

fn main():
    list = LinkedList()
    for v in [3, 1, 4, 1, 5]:
        list.push_back(v)
    list.push_front(9)
    list.show()

    list.reverse()
    list.show()
    println(f"contains 4: {list.contains(4)}, contains 7: {list.contains(7)}")

    while list.size > 3:
        if Some(v) = list.pop_front():
            println(f"popped {v}")
    list.show()

    total = sum(list.to_array())
    println(f"sum of what is left: {total}")
