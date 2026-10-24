# A13 attack: a downcast to one instance of a generic class. If the type test compared only the
# class (not its type argument), a Holder[String] would pass as a Holder[int].
open class Item:
    virtual fn name(self) -> String:
        return "item"

class Holder[T](Item):
    value: T

fn first_int(xs: Array[Item]) -> int:
    for x in xs:
        match cast x if Holder[int]:
            Some(h):
                return h.value
            None:
                pass
    return 0

fn main():
    xs: Array[Item] = [Holder[String](value="seven"), Holder[int](value=7)]
    println(first_int(xs))
