class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    bags: Array[Bag] = []
    for k in 0..2:
        b = Bag()
        for i in 0..1000:
            b.items.push(i % 7)
        bags.push(b)
    total = 0
    for round in 0..300000:
        b = bags[round % 2]
        for i in 0..1000:
            total = total + b.items[i]
    println(total)
