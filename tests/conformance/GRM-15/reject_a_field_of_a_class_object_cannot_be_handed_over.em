#$ test: compile-fail
#$ rules: GRM-15, OWN-11, EXP-6
## `[GRM-15]` — `n = owns o.name` for a field of a class object is `E3013`: the object keeps the field, and
## every handle to it can still reach it; the help writes `mem.take`.

class Obj:
    name: String = String("o")

fn main():
    o = Obj()
    n = owns o.name    #$ error[E3013]: `o` is an `Obj` object, which other names may reach, so `o.name` cannot be taken out and leave a hole
                       #$ help: `n = mem.take(ref mut o.name)`
    println(n)
