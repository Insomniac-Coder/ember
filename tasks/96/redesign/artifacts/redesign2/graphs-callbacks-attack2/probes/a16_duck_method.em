# A16 attack: Python's duck typing. A generic body calls a method that no interface declares;
# each type has its own `weight` method. Python: any object with .weight() works.
class Crate:
    kg: int = 5
    fn weight(self) -> int:
        return self.kg

class Sack:
    kg: int = 2
    fn weight(self) -> int:
        return self.kg

fn heaviest[T](xs: Array[T]) -> int:
    best = 0
    for x in xs:
        w = x.weight()
        if w > best:
            best = w
    return best

fn main():
    println(heaviest([Crate(), Crate()]))
    println(heaviest([Sack()]))
