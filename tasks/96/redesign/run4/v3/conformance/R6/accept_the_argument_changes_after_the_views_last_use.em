#$ test: run-pass
#$ rules: R6, R8
#$ stdout: key
#$ 2 1
## R6: the nearest allowed program: `w` is used before `bag` changes, so nothing it points into
## changes while it is used.

class Bag:
    items: Array[String]
    count: int = 0

fn first_item(bag: Bag) -> StrView:
    return bag.items[0]

fn main():
    bag = Bag(items=[String("key")])
    w = first_item(bag)
    println(w)
    bag.items.push(String("map"))
    bag.count += 1
    println(bag.items.len(), bag.count)
