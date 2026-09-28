#$ test: run-pass
#$ rules: IFC-4
#$ profiles: debug, release, shipping
#$ stdout: 7 0
# D-381 — a default method calls an associated function through its
# interface's associated type by its bare name (`Item.default()`), as it names
# the type elsewhere; the bound `type Item: Default` provides it. `Nums`'s
# copy calls `int`'s.

from std.core import Default

interface Source:
    type Item: Default
    fn get(self, i: int) -> Item
    fn size(self) -> int

    fn first_or_default(self) -> Item:
        if self.size() == 0:
            return Item.default()
        return self.get(0)

struct Nums:
    xs: Array[int]

extend Nums implements Source:
    type Item = int

    fn get(self, i: int) -> int:
        return self.xs[i]

    fn size(self) -> int:
        return self.xs.len()

fn main():
    println(Nums([7, 8]).first_or_default(), Nums([]).first_or_default())
