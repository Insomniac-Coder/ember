#$ test: compile-fail
#$ rules: EXC-3, DIA-2
#$ not-help: entries
# `[EXC-3]`, `[DIA-2]` (D-822) — a conflict on a `Map` field of an object names the field as the
# program writes it, `this.items`, never the map's private storage behind it.

struct Item:
    price: float
    quantity: int

class Inventory:
    items: Map[String, Item] = {}

    fn sell(name: StrView, quantity: int) -> float:
        item = ref this.items[name]
        this.items[name].quantity -= quantity    #$ error[E3021]: `this.items` is borrowed here and mutably borrowed elsewhere
        return item.price * quantity as float

fn main():
    shop = Inventory()
    shop.items["lamp"] = Item(price=2.0, quantity=3)
    println(shop.sell("lamp", 1))
