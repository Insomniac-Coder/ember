class Thing:
    a: Array[int]
    b: Array[int]
    c: Array[int]
    d: Array[int]
    e: Array[int]
    f: Array[int]
    count: int

    fn init():
        this.a = []
        this.b = []
        this.c = []
        this.d = []
        this.e = []
        this.f = []
        this.count = 0

    fn bump():
        this.count += 1

fn main():
    things: Array[Thing] = []
    for k in 0..2:
        things.push(Thing())
    for i in 0..100000000:
        t = things[i % 2]
        t.bump()
    println(things[0].count + things[1].count)
