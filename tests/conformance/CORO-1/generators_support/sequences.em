## Generators a test imports from another module (`[CORO-1]`).

pub gen fn items[T](xs: Span[T]) -> Generator[ref T, int]:
    n = 0
    for x in xs:
        n += 1
        yield x
    return n

pub struct Bag[T]:
    pub xs: Array[T]

extend[T] Bag[T]:
    pub gen fn each(self) -> Generator[ref T]:
        for x in self.xs:
            yield x
