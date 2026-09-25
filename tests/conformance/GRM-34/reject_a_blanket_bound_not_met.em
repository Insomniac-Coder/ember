#$ test: compile-fail
#$ rules: GRM-34, TYP-17
# ODR-042 — a blanket implementation covers only the arguments its bounds
# admit: `Label` is `Tag[T]` for a `T: Display`, and `Point` has no `Display`.

interface Tag[T]:
    fn tag(self, t: T) -> String

struct Label:
    name: String

struct Point:
    x: int

extend[T: Display] Label implements Tag[T]:
    fn tag(self, t: T) -> String:
        return f"{self.name}={t}"

fn show[L: Tag[Point]](l: L) -> String:
    return l.tag(Point(x=1))

fn main():
    l = Label(name=String.from("n"))
    println(show(l))    #$ error[E2040]: `Label` does not implement `Tag[Point]`, which `L` requires
