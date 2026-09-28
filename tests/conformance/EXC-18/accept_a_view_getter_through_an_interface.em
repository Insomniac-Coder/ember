#$ test: run-pass
#$ rules: EXC-18, LT-1, BRW-8, TYP-22, OBJ-2
#$ profiles: debug, release, shipping
#$ stdout: ann
#$ stdout: ann
#$ stdout: bob
# `[LT-1]` rule 1 — a view-returning method takes its class receiver by address,
# however it is called: directly, through an interface-typed handle, or through a
# `dyn` box, whose adapter passes the receiver the same way (F3: it crashed).

interface Named:
    fn name(self) -> str

class Person implements Named:
    label: String

    fn init(mut self):
        self.label = "ann"

    fn name(self) -> str:
        return self.label

    fn rename(mut self):
        self.label = "bob"

fn show(n: Named):
    println(n.name())

fn main():
    p = Person()
    show(p)
    alias = p
    boxed: Box[dyn Named] = Box(alias)
    println(boxed.name())
    p.rename()
    println(boxed.name())
