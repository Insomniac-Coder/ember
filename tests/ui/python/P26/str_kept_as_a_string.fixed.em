struct Item:
    name: String

fn make(name: StrView) -> Item:
    return Item(name=name.to_string())

fn main():
    println(make("lamp").name)
