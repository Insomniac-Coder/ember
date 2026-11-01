class Bag:
    items: Array[int]

    fn init():
        this.items = []

fn main():
    bags: Array[Bag] = []
    for k in 0..6:
        b = Bag()
        b.items.push(owns k)
        bags.push(owns b)
    s0 = bags[2].items.as_span()
    s1 = bags[3].items.as_span()
    s2 = bags[4].items.as_span()
    s3 = bags[5].items.as_span()
    total = 0
    for round in 0..100000:
        sink = bags[round % 2]
        sink.items.clear()
        for i in 0..1000:
            sink.items.push(owns i)
        total = total + len(sink.items)
    println(total, s0[0] + s1[0] + s2[0] + s3[0])
