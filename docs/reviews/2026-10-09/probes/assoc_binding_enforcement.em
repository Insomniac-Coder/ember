interface Element:
    type Item

interface Holder:
    type Value: Element[Item = int]

struct Wrong:
    n: int

extend Wrong implements Element:
    type Item = str

struct Owner:
    n: int

extend Owner implements Holder:
    type Value = Wrong

fn main():
    pass
