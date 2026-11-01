import std.mem
from std.core import Default

struct Item:
    pub value: i32

    fn drop():
        pass

extend Item implements Default:
    static fn default() -> Item:
        return Item(0)

struct Holder:
    pub item: Item

    fn drop():
        old = mem.take(ref mut this.item)
        println(old.value)

fn main():
    _holder = Holder(Item(7))
