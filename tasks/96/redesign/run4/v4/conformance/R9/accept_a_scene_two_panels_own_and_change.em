#$ test: compile-pass
#$ rules: R9, R5
## R9: the nearest allowed program to `reject_refcell_is_not_ember.em`: a scene several objects own and
## change is held `shares mut` by each, and changed by a method whose first line says so (AN-49).
## `main` changes the scene through the panels' links, so it says `changes Scene` (R5, verdict 64).

struct Scene:
    entities: Array[int]

    fn add(e: int) changes entities:
        this.entities.push(e)

class Panel:
    scene: shares mut Scene

fn main() changes Scene:
    s = Scene(entities=[])
    left = Panel(scene=shares mut s)
    right = Panel(scene=shares mut s)
    left.scene.add(1)
    right.scene.add(2)
    println(s.entities.len())
