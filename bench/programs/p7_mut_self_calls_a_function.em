class Thing:
    a: Array[int]
    b: Array[int]
    c: Array[int]
    d: Array[int]
    e: Array[int]
    f: Array[int]
    count: int

    fn init(mut self):
        self.a = []
        self.b = []
        self.c = []
        self.d = []
        self.e = []
        self.f = []
        self.count = 0

    fn bump(mut self):
        self.count = step(self.count)

fn step(x: int) -> int:
    return x + 1

fn main():
    things: Array[Thing] = []
    for k in 0..2:
        things.push(Thing())
    for i in 0..100000000:
        t = things[i % 2]
        t.bump()
    println(things[0].count + things[1].count)
