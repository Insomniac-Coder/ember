class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    bags: Array[Bag] = []
    for k in 0..3:
        b = Bag()
        b.items.push(k)
        bags.push(b)
    s0 = bags[2].items.as_span()
    total = 0
    for round in 0..100000:
        sink = bags[round % 2]
        sink.items.clear()
        for i in 0..1000:
            sink.items.push(i)
        total = total + len(sink.items)
    println(total, s0[0])
