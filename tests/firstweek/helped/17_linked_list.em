# A singly linked list of integers built from class nodes: push at both ends, pop, reverse, search and print.

class Node:
    value: int
    next: Option[shares mut Node] = None

    fn init(value: int):
        this.value = value

fn last_of(owns node: shares mut Node) -> shares mut Node:
    if node.next exists as n:
        return last_of(shares mut n)
    return node

class LinkedList:
    head: Option[shares mut Node] = None
    size: int = 0

    fn push_front(value: int):
        node = shares mut Node(value)
        node.next = this.head.take()
        this.head = Some(node)
        this.size += 1

    fn pop_front() -> Option[int]:
        if this.head.take() exists as node:
            this.head = node.next.take()
            this.size -= 1
            return Some(node.value)
        return None

    fn push_back(value: int):
        node = shares mut Node(value)
        if this.head exists as first:
            last = last_of(shares mut first)
            last.next = Some(node)
        else:
            this.head = Some(node)
        this.size += 1

    fn contains(value: int) -> bool:
        current = ref this.head
        while current exists as node:
            if node.value == value:
                return true
            current = ref node.next
        return false

    fn reverse():
        prev: Option[shares mut Node] = None
        current = this.head.take()
        while current is not None:
            node = current.unwrap()
            current = node.next.take()
            node.next = prev
            prev = Some(node)
        this.head = prev

    fn to_array() -> Array[int]:
        result: Array[int] = []
        current = ref this.head
        while current exists as node:
            result.push(owns node.value)
            current = ref node.next
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
