#$ test: run-pass
#$ rules: GRM-34, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: n=5 n=x n=1.5
#$ n=5
# ODR-042 — an `extend` parameter that only the implemented interface names
# makes a blanket implementation: `Label` implements `Tag[T]` for every
# `T: Display`, and `tag` is generic over it.

interface Tag[T]:
    fn tag(self, t: T) -> String

struct Label:
    name: String

extend[T: Display] Label implements Tag[T]:
    fn tag(self, t: T) -> String:
        return f"{self.name}={t}"

fn show[L: Tag[int]](l: L) -> String:
    return l.tag(5)

fn main():
    l = Label(name=String.from("n"))
    println(l.tag(5), l.tag("x"), l.tag(1.5))
    println(show(l))
