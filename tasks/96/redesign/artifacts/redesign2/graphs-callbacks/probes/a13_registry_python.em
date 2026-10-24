# A13: the registry a Python programmer writes: pools found by their type, no cast.
struct Pool[T]:
    dense: Array[T] = []

struct Registry:
    pools: Map[type, Pool]

    fn get[T](self, i: int) -> T:
        return self.pools[T].dense[i]

fn main():
    r = Registry(pools=Map())
    println(r.get[int](0))
