#$ test: compile-fail
#$ rules: OWN-11

struct Inventory:
    items: Array[int]

fn main():
    inv = shares mut Inventory(items=[1])
    lists = [[0]]
    lists.push(owns inv.items)    #$ error[E3013]
    println(lists.len())
