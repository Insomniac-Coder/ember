#$ test: run-pass
#$ rules: STR-5, OWN-8
#$ stdout: ['a'] ['a', 'b']
#$ n
#$ 101
#$ 1
# `[STR-5]` — a struct or enum implements `Clone` field-wise, with nothing
# written, when every field does. A hand-written `clone` replaces the
# implicit one. A type with a field that cannot be cloned is simply not
# `Clone`: that is an error only where a clone is asked for.

struct Point:
    x: int
    tags: Array[String]

enum Shape:
    Circle(int)
    Named(String)

struct Custom:
    n: int

    fn clone(self) -> Custom:
        return Custom(n=self.n + 100)

struct Resource:
    id: int

    fn drop(mut self):
        pass

struct Holder:
    r: Resource

fn main():
    p = Point(x=1, tags=["a"])
    q = p.clone()
    q.tags.push("b")
    println(p.tags, q.tags)
    s = Shape.Named("n")
    match s.clone():
        Shape.Named(name):
            println(name)
        Shape.Circle(r):
            println(r)
    c = Custom(n=1)
    println(c.clone().n)
    h = Holder(r=Resource(1))
    println(h.r.id)
