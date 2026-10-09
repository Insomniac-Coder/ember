interface Holder:
    type Item: Ord

fn bad[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a < b

fn main():
    pass
