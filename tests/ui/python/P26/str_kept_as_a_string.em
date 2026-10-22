struct Item:
    name: String

fn make(name: str) -> Item:
    return Item(name=name)

fn main():
    println(make("lamp").name)
