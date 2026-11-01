class Bag:
    items: Array[int]

fn main():
    b = Bag([1, 2, 3])
    view = b.items[..]
    b.items.push(owns 4)
    println(view)
