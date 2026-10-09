interface Holder:
    type Item

fn bad[H: Holder](a: H.Item, b: H.Item) -> bool:
    return a < b

fn main():
    pass
