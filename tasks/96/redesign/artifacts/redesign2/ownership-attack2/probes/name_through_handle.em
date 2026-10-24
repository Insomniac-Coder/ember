# name_through_handle.em: a name for a list inside an object, then the handle variable rebound.
# Python: n keeps naming the old object's list.
class Player:
    items: Array[int] = []

fn main():
    p = Player()
    n = p.items
    p = Player()
    n.push(1)
    println(len(n))
