# gen_push_between.em: a generator walks a list it was given; between two next() calls the caller
# appends to that list. Python: the generator sees the appended item.
gen fn walk(xs: Array[int]) -> Generator[int]:
    for x in xs:
        yield x

fn main():
    items = [1, 2]
    g = walk(items)
    println(g.next())
    items.push(99)
    println(g.next())
    println(g.next())
