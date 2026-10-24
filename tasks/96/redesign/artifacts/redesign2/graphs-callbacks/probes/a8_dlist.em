# A8: a doubly linked list; Python writes strong next and prev.
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
