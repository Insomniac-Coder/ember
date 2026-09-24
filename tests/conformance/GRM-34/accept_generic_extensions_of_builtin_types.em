#$ test: run-pass
#$ rules: GRM-34, TYP-24
#$ stdout:
#$ 2 102 [name 1;name 2;]
#$ true name 5 none
# `[GRM-34]` — `extend[T: B] Array[T] implements I:` is the rule's own
# example, and `Option` and `Result` extend the same way. `Array`'s own `len`
# comes first (`[TYP-24]`), so the extension's `self.len()` is not itself,
# and `Sized.len(xs)` names the interface's.

interface Sized:
    fn len(self) -> int

interface Describe:
    fn describe(self) -> String

interface Check:
    fn good(self) -> bool

struct Name:
    code: int

extend Name implements Describe:
    fn describe(self) -> String:
        return f"name {self.code}"

extend[T] Array[T] implements Sized:
    fn len(self) -> int:
        return self.len() + 100

extend[T: Describe] Array[T] implements Describe:
    fn describe(self) -> String:
        out = String.from("[")
        for x in self:
            out += x.describe()
            out += ";"
        out += "]"
        return out

extend[T: Describe] Option[T] implements Describe:
    fn describe(self) -> String:
        match self:
            Some(x): return x.describe()
            None: return String.from("none")

extend[T, E] Result[T, E] implements Check:
    fn good(self) -> bool:
        return self.is_ok()

fn show[D: Describe](d: D) -> String:
    return d.describe()

fn tell(d: ref dyn Describe) -> String:
    return d.describe()

xs = [Name(1), Name(2)]
r: Result[int, str] = Ok(3)
some = Some(Name(5))
none: Option[Name] = None
println(xs.len(), Sized.len(xs), show(xs))
println(r.good(), tell(ref some), show(none))
