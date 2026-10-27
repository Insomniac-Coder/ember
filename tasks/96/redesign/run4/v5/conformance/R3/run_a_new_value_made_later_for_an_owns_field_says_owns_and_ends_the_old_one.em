#$ test: run-pass
#$ rules: R3, R2
#$ stdout: end a
#$ replaced
#$ end b
## R3, R2: `e.mesh = owns Mesh(...)` gives `e` a new mesh in a block of its own, and the old mesh ends
## there, once (verdict 97; R2's "`=` puts the new value in the place").

class Res:
    name: String

    fn drop():
        println("end", this.name)

struct Mesh:
    res: Res

struct Entity:
    mesh: owns Mesh

fn main():
    e = Entity(mesh=owns Mesh(res=Res(name=String("a"))))
    e.mesh = owns Mesh(res=Res(name=String("b")))
    println("replaced")
