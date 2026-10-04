#$ test: run-pass
#$ rules: WK-4, WK-15, WK-8
#$ profiles: debug
#$ warning[L3001]: potential reference cycle: Item.owner -> Holder.items -> Item
#$ stdout: built
#$ stderr: warning[L3017]: reference cycle detected
#$ stderr: shortest cycles:
#$ stderr: Holder.items -> Item.owner: Holder@
#$ stderr: Item.owner -> Holder.items: Item@
# `[WK-4]` — the report names, for each leaked object on a cycle, the shortest
# strong cycle through it as `Type.field` edges. The holder's runs through its
# list into an item and back; each item's runs through its owner and back, and
# the two items share that shape, so it is given once with both.

class Item:
    owner: Option[Holder]

class Holder:
    items: Array[Item]

fn main():
    holder = Holder(Array[Item]())
    first = Item(Some(holder))
    second = Item(Some(holder))
    holder.items.push(first)
    holder.items.push(second)
    println("built")
