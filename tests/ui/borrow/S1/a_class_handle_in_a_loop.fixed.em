struct Bag:
    items: Array[int]

@static_safe
fn fill(mut b: Bag):
    for i in 0..3:
        b.items.push(i)

fn main():
    b = Bag([])
    fill(b)
    for x in b.items:
        println(x)
