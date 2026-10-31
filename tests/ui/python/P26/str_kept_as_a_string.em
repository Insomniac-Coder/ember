struct Item:
    name: String

fn make(name: StrView) -> Item:
    return Item(name=name)

fn main():
    println(make("lamp").name)
