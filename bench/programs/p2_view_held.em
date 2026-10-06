class Bag:
    items: Array[int]
    log: Array[int]

    fn init(mut self):
        self.items = []
        self.log = []

fn main():
    bags: Array[Bag] = []
    for k in 0..2:
        b = Bag()
        for i in 0..1000:
            b.items.push(i)
        bags.push(b)
    total = 0
    for round in 0..100000:
        b = bags[round % 2]
        b.log.clear()
        for x in b.items:
            b.log.push(x)
        total = total + len(b.log)
    println(total)
