#$ test: compile-fail
#$ rules: RC-1, OWN-13
# `[RC-1]` — a field has one owner, the object holding it, and no second owner is made of one field
# (`E2122`, `shares this.items`): the field is made to share its value from the start.

class Item:
    v: int = 1

class Bag:
    item: Item

    fn lend() -> shares Item:
        return shares this.item       #$ error[E2122]: `this.item` has one owner, the `Bag` that holds it, and no second owner can be made of one field; to let other names own the item, make the field share it from the start: write `item: shares Item` in `Bag` (line 10)

fn main():
    b = Bag(item=Item())
    println(b.lend().v)
