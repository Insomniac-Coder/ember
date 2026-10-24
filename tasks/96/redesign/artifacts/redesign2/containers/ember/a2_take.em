# A2, removal: name an item, remove it from the list, keep using it (Python: `picked` keeps the object).
struct Item:
    name: String
    weight: int

fn pick(mut items: Array[Item], mut bag: Array[Item], i: int):
    picked = items[i]
    items.remove(i)
    bag.push(picked)

fn main():
    items = [Item(name="sword", weight=3), Item(name="shield", weight=5)]
    bag: Array[Item] = []
    pick(items, bag, 0)
    println(bag.len())
