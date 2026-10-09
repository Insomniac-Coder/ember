interface Tag[T]:
    fn tag(self) -> int

interface Outer[T]:
    type Member: Tag[int]

struct Value:
    n: int

extend Value implements Tag[int]:
    fn tag(self) -> int:
        return self.n

struct Owner:
    n: int

extend Owner implements Outer[int]:
    type Member = Value

fn main():
    pass
