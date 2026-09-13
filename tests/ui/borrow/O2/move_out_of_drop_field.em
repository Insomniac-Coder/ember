from std.core import Default

struct Item:
    pub value: i32

    fn drop(mut self):
        pass

extend Item implements Default:
    fn default() -> Item:
        return Item(0)

struct Holder:
    pub item: Item

    fn drop(mut self):
        old = self.item
        println(old.value)

fn main():
    _holder = Holder(Item(7))
