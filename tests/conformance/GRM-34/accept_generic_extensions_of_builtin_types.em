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
    fn len() -> int

interface Describe:
    fn describe() -> String

interface Check:
    fn good() -> bool

struct Name:
    code: int

extend Name implements Describe:
    fn describe() -> String:
        return f"name {this.code}"

extend[T] Array[T] implements Sized:
    fn len() -> int:
        return this.len() + 100

extend[T: needs Describe] Array[T] implements Describe:
    fn describe() -> String:
        out = String.from("[")
        for x in this:
            out += x.describe()
            out += ";"
        out += "]"
        return out

extend[T: needs Describe] Option[T] implements Describe:
    fn describe() -> String:
        match this:
            Some(x): return x.describe()
            None: return String.from("none")

extend[T, E] Result[T, E] implements Check:
    fn good() -> bool:
        return this.is_ok()

[D: needs Describe]
fn show(d: D) -> String:
    return d.describe()

fn tell(d: ref dyn Describe) -> String:
    return d.describe()

xs = [Name(1), Name(2)]
r: Result[int, StrView] = Ok(3)
some = Some(Name(5))
none: Option[Name] = None
println(xs.len(), Sized.len(xs), show(xs))
println(r.good(), tell(ref some), show(none))
