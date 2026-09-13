struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

fn consume(owned pair: Pair):
    println(pair.second.n)

fn main():
    pair = Pair(Part(1), Part(2))
    first = pair.first
    println(pair.second.n)
    pair.first = first
    consume(pair)

