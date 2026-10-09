interface Lookalike:
    fn add(self, rhs: Self) -> Self

interface Holder:
    type Item: Lookalike

fn bad[H: Holder](a: H.Item, b: H.Item) -> H.Item:
    return a + b

fn main():
    pass
