#$ test: compile-fail
#$ rules: BRW-1
#$ help: bag.count += 1
## design rules: R6, R8
## R6: a result that points into an argument points into all of it, as Rust's borrow of the whole
## argument does: a change of another field of `bag` while the view is used is refused (R8 (g),
## E3021).

class Bag:
    items: Array[String]
    count: int = 0

fn first_item(bag: Bag) -> StrView:
    return bag.items[0]

fn main():
    bag = Bag(items=[String("key")])
    w = first_item(bag)
    bag.count += 1    #$ error[E3021]: `bag.count += 1` (line 18) changes `bag`, which `w` points into (`first_item(bag)`, line 17: a function's result points into all of its view argument), and `w` is used on line 19; change `bag` after `w`'s last use: move `bag.count += 1` below line 19
    println(w, bag.count)
