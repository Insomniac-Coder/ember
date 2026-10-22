#$ test: compile-fail
#$ rules: EXC-3, DIA-2
#$ not-help: entries
# `[EXC-3]`, `[DIA-2]` (D-822) — a conflict on a `Map` field of an object names the field as the
# program writes it, `self.items`, never the map's private storage behind it.

struct Item:
    price: float
    quantity: int

class Inventory:
    items: Map[String, Item] = {}

    fn sell(self, name: str, quantity: int) -> float:
        item = ref self.items[name]
        self.items[name].quantity -= quantity    #$ error[E3080]: `self.items` is borrowed here and mutably borrowed elsewhere
        return item.price * quantity as float

fn main():
    shop = Inventory()
    shop.items["lamp"] = Item(price=2.0, quantity=3)
    println(shop.sell("lamp", 1))
