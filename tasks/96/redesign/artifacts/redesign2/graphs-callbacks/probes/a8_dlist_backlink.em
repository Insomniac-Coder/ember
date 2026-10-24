# A8, proposed: a doubly linked list; `prev` is the back link of `next` (one owner each).
class Item:
    value: int
    next: Option[Item] = None
    prev: Option[Item] = backlink(next)

fn main():
    a = Item(1)
    b = Item(2)
    a.next = Some(b)                  # also sets b.prev
    println(b.prev.unwrap().value)
