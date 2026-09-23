open class Base:
    value: i32

class Derived(Base):
    current: i32

fn read(item: Derived) -> i32:
    return item.value

fn main():
    println(0)
