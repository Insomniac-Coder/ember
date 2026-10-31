## Generators a test imports from another module (`[CORO-1]`).

[T]
pub gen fn items(xs: Span[T]) -> Generator[ref T, int]:
    n = 0
    for x in xs:
        n += 1
        yield x
    return n

[T]
pub struct Bag:
    pub xs: Array[T]

extend[T] Bag[T]:
    pub gen fn each() -> Generator[ref T]:
        for x in this.xs:
            yield x
