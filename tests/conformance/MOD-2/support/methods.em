## Methods of every visibility, for `reject_a_private_method_from_another_module.em`
## and `accept_a_pub_method_and_an_interface_method_anywhere.em`.

pub interface Shown:
    fn shown() -> int

interface Hidden:
    fn hidden() -> int

pub struct Meter:
    pub value: int

    pub fn read() -> int:
        return this.twice() // 2

    fn twice() -> int:
        return this.value * 2

    pub static fn make(v: int) -> Meter:
        return Meter(v)

    static fn raw(v: int) -> Meter:
        return Meter(v)

extend Meter implements Shown:
    fn shown() -> int:
        return this.value + 1

extend Meter implements Hidden:
    fn hidden() -> int:
        return this.value + 2

[T]
pub struct Box2:
    pub item: T

    pub fn get() -> T:
        return this.item

    fn peek() -> T:
        return this.item
