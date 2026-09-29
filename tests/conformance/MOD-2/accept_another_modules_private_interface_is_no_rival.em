#$ test: run-pass
#$ rules: MOD-2, TYP-24
#$ stdout: 70 42
# `[MOD-2]`, D-384 — an interface private to another module offers nothing
# here: `Meter` implements `support.methods`' private `Hidden`, whose `hidden`
# neither answers `m.hidden()` here nor makes it ambiguous (`E2070`) beside
# this module's own `Mine`. So is std's private `Integer`'s method beside a
# program's own of the same name.

from support.methods import Meter

interface Mine:
    fn hidden(self) -> int

extend Meter implements Mine:
    fn hidden(self) -> int:
        return self.value * 10

interface Next:
    fn successor(self) -> int

extend int implements Next:
    fn successor(self) -> int:
        return self * 2

fn main():
    m = Meter.make(7)
    println(m.hidden(), 21.successor())
