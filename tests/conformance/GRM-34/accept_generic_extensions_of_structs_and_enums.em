#$ test: run-pass
#$ rules: GRM-34, IFC-1
#$ stdout:
#$ 9 true false
#$ a name nothing 5
# `[GRM-34]` — `extend[T]` extends any generic type, not only a class: a
# struct, an enum, inherently or with an interface under a bound. D-242: an
# instance named before the extensions are read (here, in an interface's
# signature) gets them too.

interface Describe:
    fn describe(self) -> str

interface Maker:
    fn make(self) -> Holder[i64]

struct Holder[T]:
    value: T

extend[T] Holder[T]:
    fn get(self) -> T:
        return self.value

enum Maybe[T]:
    Nothing
    Just(T)

extend[T] Maybe[T]:
    fn has(self) -> bool:
        match self:
            Maybe.Nothing: return false
            Maybe.Just(_): return true

extend[U: Describe] Maybe[U] implements Describe:
    fn describe(self) -> str:
        match self:
            Maybe.Nothing: return "nothing"
            Maybe.Just(x): return x.describe()

struct Name:
    code: int

extend Name implements Describe:
    fn describe(self) -> str:
        return "a name"

struct Factory implements Maker:
    fn make(self) -> Holder[i64]:
        return Holder(5)

fn show[D: Describe](d: D) -> str:
    return d.describe()

println(Holder(9).get(), Maybe[Name].Just(Name(7)).has(), Maybe[int].Nothing.has())
println(show(Maybe[Name].Just(Name(7))), show(Maybe[Name].Nothing), Factory().make().get())
