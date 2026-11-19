class Bag:
    items: Array[int]

    fn add(x: int):
        this.items.push(owns x)

@no_alias_checks
fn fill(ref mut b: Bag):
    for i in 0..3:
        b.add(i)

fn main():
    b = Bag([])
    c = b
    fill(ref mut b)
    for x in c.items:
        println(x)
