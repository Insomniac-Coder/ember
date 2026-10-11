class Bag:
    items: Array[int]

    fn add(mut self, x: int):
        self.items.push(x)

@static_safe
fn fill(mut b: Bag):
    for i in 0..3:
        b.add(i)

fn main():
    b = Bag([])
    c = b
    fill(b)
    for x in c.items:
        println(x)
