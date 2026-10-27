#$ test: compile-pass
#$ rules: MOD-3
## design rules: R9, R5
## R9: the nearest allowed program to `reject_refcell_is_not_ember.em`: a scene several objects own and
## change is held `shares mut` by each, and changed in a method, `add` (AN-49).

struct Scene:
    entities: Array[int]

    fn add(e: int):
        this.entities.push(e)

class Panel:
    scene: shares mut Scene

fn main():
    s = Scene(entities=[])
    left = Panel(scene=shares mut s)
    right = Panel(scene=shares mut s)
    left.scene.add(1)
    right.scene.add(2)
    println(s.entities.len())
