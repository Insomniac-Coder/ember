# A singly linked list of integers built from class nodes: push at both ends, pop, reverse, search and print.

class Node:
    value: int
    next: Option[Node] = None

    fn init(value: int):
        this.value = value

class LinkedList:
    head: Option[Node] = None
    size: int = 0

    fn push_front(value: int):
        node = Node(value)
        node.next = this.head
        this.head = Some(node)
        this.size += 1

    fn pop_front() -> Option[int]:
        if this.head exists as node:
            this.head = node.next
            this.size -= 1
            return Some(node.value)
        return None

    fn push_back(value: int):
        node = Node(value)
        if this.head is None:
            this.head = Some(node)
        else:
            current = this.head.unwrap()
            while current.next exists as n:
                current = n
            current.next = Some(node)
        this.size += 1

    fn contains(value: int) -> bool:
        current = this.head
        while current exists as node:
            if node.value == value:
                return true
            current = node.next
        return false

    fn reverse():
        prev: Option[Node] = None
        current = this.head
        while current exists as node:
            current = node.next
            node.next = prev
            prev = Some(node)
        this.head = prev

    fn to_array() -> Array[int]:
        result: Array[int] = []
        current = this.head
        while current exists as node:
            result.push(node.value)
            current = node.next
        return result

    fn show():
        parts = [v.to_string() for v in this.to_array()]
        println(parts.join(" -> ") + f"  (size {this.size})")

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
        if list.pop_front() exists as v:
            println(f"popped {v}")
    list.show()

    total = sum(list.to_array())
    println(f"sum of what is left: {total}")
