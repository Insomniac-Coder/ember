## Methods of every visibility, for `reject_a_private_method_from_another_module.em`
## and `accept_a_pub_method_and_an_interface_method_anywhere.em`.

pub interface Shown:
    fn shown(self) -> int

interface Hidden:
    fn hidden(self) -> int

pub struct Meter:
    pub value: int

    pub fn read(self) -> int:
        return self.twice() // 2

    fn twice(self) -> int:
        return self.value * 2

    pub fn make(v: int) -> Meter:
        return Meter(v)

    fn raw(v: int) -> Meter:
        return Meter(v)

extend Meter implements Shown:
    fn shown(self) -> int:
        return self.value + 1

extend Meter implements Hidden:
    fn hidden(self) -> int:
        return self.value + 2

pub struct Box2[T]:
    pub item: T

    pub fn get(self) -> T:
        return self.item

    fn peek(self) -> T:
        return self.item
