# gen_store_between.em: the same generator; between two next() calls the caller stores the list into
# a field of an object (a move today; under "names follow values" the name `items` follows it).
class World:
    items: Array[int] = []

gen fn walk(xs: Array[int]) -> Generator[int]:
    for x in xs:
        yield x

fn main():
    items = [1, 2]
    w = World()
    g = walk(items)
    println(g.next())
    w.items = items
    w.items.push(99)
    println(g.next())
