# Clause CLS-12.1: "`@cached` above a class method with no parameters keeps the method's result
# after the first call, in the object (...), and gives it again until a change, through any name,
# of a field the method's body reads clears it (`[EFF-27]`). A `Copy` result is returned by value,
# any other as a `ref T` view into the kept value"
#$ test: run-pass
#$ rules: CLS-12, EFF-27, EFF-24
#$ stdout: 3.0 3.0
#$ stdout: 7.0
#$ stdout: 9.0
#$ stdout: 10.0
#$ stdout: ann:2
#$ stdout: bob:2
# Each change of `verts`, through the object's name, a `ref mut` parameter and a method, clears the
# kept total; a change of `tag`, which `total` does not read, keeps it. `label`'s `String` is given
# as a `ref String` view.

class Mesh:
    verts: Array[float] = []
    tag: int = 0
    name: String

    fn init(owns name: String):
        this.name = name

    @cached
    fn total() -> float:
        t = 0.0
        for v in this.verts:
            t += v
        return t

    @cached
    fn label() -> String:
        return f"{this.name}:{this.verts.len()}"

    fn add(v: float):
        this.verts.push(owns v)

fn push_two(ref mut m: Mesh):
    m.verts.push(owns 1.0)
    m.verts.push(owns 1.0)

fn main():
    m = Mesh(String("ann"))
    m.verts.push(owns 1.0)
    m.verts.push(owns 2.0)
    println(m.total(), m.total())
    m.verts.push(owns 4.0)
    m.tag = 5
    println(m.total())
    push_two(ref mut m)
    println(m.total())
    m.add(1.0)
    println(m.total())
    m.verts = [1.0, 2.0]
    println(m.label())
    m.name = String("bob")
    println(m.label())
