class Thing:
    a: Array[int]
    count: int

    fn init(mut self):
        self.a = []
        self.count = 0

    fn bump(mut self):
        self.count += 1

fn main():
    things: Array[Thing] = []
    for k in 0..2:
        things.push(Thing())
    for i in 0..100000000:
        t = things[i % 2]
        t.bump()
    println(things[0].count + things[1].count)
