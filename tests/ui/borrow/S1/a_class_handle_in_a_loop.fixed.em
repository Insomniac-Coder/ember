struct Bag:
    items: Array[int]

@no_alias_checks
fn fill(ref mut b: Bag):
    for i in 0..3:
        b.items.push(i)

fn main():
    b = Bag([])
    fill(b)
    for x in b.items:
        println(x)
