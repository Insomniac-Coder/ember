#$ test: run-pass
#$ rules: IFC-2, TYP-16
#$ stdout: 7 0
#$ stdout: true

## D-728 — an interface method whose signature names a `Map` (or `Set`) instance made that instance
## while interfaces were still being collected, and its methods' bounds were kept unsubstituted:
## `Map`'s own `get_or` then failed, "`Q` does not implement `AsKey[Q]`", in `std`, with an empty
## `main`. The bound is named over its arguments now and instantiated once interfaces are known.

interface Ops:
    type Fetch
    fn fetch(storages: Map[u64, int], key: u64) -> Fetch
    fn seen(ids: Set[u64], id: u64) -> bool

struct Store:
    n: int

extend Store implements Ops:
    type Fetch = int
    fn fetch(storages: Map[u64, int], key: u64) -> int:
        return storages.get_or(key, 0)
    fn seen(ids: Set[u64], id: u64) -> bool:
        return ids.contains(id)

fn main():
    m: Map[u64, int] = {3: 7}
    println(Store.fetch(m, 3), Store.fetch(m, 4))
    s: Set[u64] = {5}
    println(Store.seen(s, 5))
