class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    bags: Array[Bag] = []
    for k in 0..18:
        b = Bag()
        b.items.push(k)
        bags.push(b)
    s0 = bags[2].items.as_span()
    s1 = bags[3].items.as_span()
    s2 = bags[4].items.as_span()
    s3 = bags[5].items.as_span()
    s4 = bags[6].items.as_span()
    s5 = bags[7].items.as_span()
    s6 = bags[8].items.as_span()
    s7 = bags[9].items.as_span()
    s8 = bags[10].items.as_span()
    s9 = bags[11].items.as_span()
    s10 = bags[12].items.as_span()
    s11 = bags[13].items.as_span()
    s12 = bags[14].items.as_span()
    s13 = bags[15].items.as_span()
    s14 = bags[16].items.as_span()
    s15 = bags[17].items.as_span()
    total = 0
    for round in 0..100000:
        sink = bags[round % 2]
        sink.items.clear()
        for i in 0..1000:
            sink.items.push(i)
        total = total + len(sink.items)
    println(total, s0[0] + s1[0] + s2[0] + s3[0] + s4[0] + s5[0] + s6[0] + s7[0] + s8[0] + s9[0] + s10[0] + s11[0] + s12[0] + s13[0] + s14[0] + s15[0])
