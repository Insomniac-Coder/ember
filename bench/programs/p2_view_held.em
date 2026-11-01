class Bag:
    items: Array[int]
    log: Array[int]

    fn init():
        this.items = []
        this.log = []

fn main():
    bags: Array[Bag] = []
    for k in 0..2:
        b = Bag()
        for i in 0..1000:
            b.items.push(owns i)
        bags.push(owns b)
    total = 0
    for round in 0..100000:
        b = bags[round % 2]
        b.log.clear()
        for x in b.items:
            b.log.push(owns x)
        total = total + len(b.log)
    println(total)
