#$ test: run-pass
#$ rules: CLS-5
#$ profiles: debug, development
#$ stdout: 3 3 40
#$ assert-c: contains(= em_3Dog5speak()
#$ assert-c: !contains(em_vt_dyn_5Speak_for_3Dog)
#$ assert-c: contains(->slot0)()
# `[CLS-5]` — an interface method called on a class is a direct call, both on a receiver of the
# class's own type and through a generic bound (the instance knows the class); only a `dyn`
# receiver goes through the interface's table. So `Dog` gets no table adapter at all, and the
# boxed `Cat`'s call is the one call through a slot.

interface Speak:
    fn speak(self) -> int

class Dog implements Speak:
    n: int

    fn speak(self) -> int:
        return self.n

class Cat implements Speak:
    n: int

    fn speak(self) -> int:
        return self.n * 10

fn via_generic[T: Speak](x: T) -> int:
    return x.speak()


fn main():
    d = Dog(3)
    c = Cat(4)
    boxed: Box[dyn Speak] = Box(c)
    println(d.speak(), via_generic(d), boxed.speak())
