#$ test: compile-fail
#$ rules: GRM-15, OWN-11, EXP-6
## `[GRM-15]` — the refusals of `body = owns part`: a field of a type with `drop` is `E3010`, an element
## `E3011`, a field of a class object or a part reached through a name that does not own it `E3013`.

struct Guard:
    name: String = String("g")

    fn drop():
        println("drop")

class Obj:
    name: String = String("o")

struct Req:
    body: Array[int] = []

fn viewer(r: Req):
    b = owns r.body    #$ error[E3013]: cannot move out of a borrowed parameter
    println(b.len())

fn main():
    g = Guard()
    n1 = owns g.name    #$ error[E3010]: cannot hand over `g.name` on its own
    xs = [String("a")]
    n3 = owns xs[0]    #$ error[E3011]: cannot move out of an element
    println(n1, n3)
