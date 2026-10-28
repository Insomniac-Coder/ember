#$ test: compile-fail
#$ rules: R6, R8
#$ help: println(w)
## R6: pointing into an argument means pointing into everything it owns, so a reshape of a list `bag`
## owns ends the view (R8 (b), E3060).

class Bag:
    items: Array[String]
    count: int = 0

fn first_item(bag: Bag) -> StrView:
    return bag.items[0]

fn main():
    bag = Bag(items=[String("key")])
    w = first_item(bag)
    bag.items.push(String("map"))
    println(w)    #$ error[E3060]: `bag.items.push(String("map"))` (line 17) may move the list that `w` points into (`first_item(bag)`, line 16), and `w` is used on line 18; use `w` before line 17: move `println(w)` above line 17
