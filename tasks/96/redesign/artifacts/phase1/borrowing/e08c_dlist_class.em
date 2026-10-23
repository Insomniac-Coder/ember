# A8: a doubly linked list of class nodes with strong `next` and strong `prev` (a cycle).
class Item:
    value: int
    next: Option[Item] = None
    prev: Option[Item] = None

fn main():
    a = Item(1)
    b = Item(2)
    a.next = Some(b)
    b.prev = Some(a)
    println(a.value)
