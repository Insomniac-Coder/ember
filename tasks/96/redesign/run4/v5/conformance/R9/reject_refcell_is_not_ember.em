#$ test: compile-fail
#$ rules: R9
#$ note: a change is worked out from the code that makes it, and checked when compiling
#$ help: write the field plainly, `entities: Array[int]`, and change it in a method: `fn add(e: int):` with the body `this.entities.push(e)`; for a scene several objects own and change, make it once, `s = Scene(entities=[])`, write the field that keeps it in each owner as `scene: shares mut Scene`, and give each owner its share where it is made, for example `Panel(scene=shares mut s)`
## R9: "`RefCell`, `Ref`, `RefMut`, `borrow` and `borrow_mut` are removed": nothing is checked while
## running (AN-49, still refused).

struct Scene:
    entities: RefCell[Array[int]]    #$ error[E1010]: `RefCell` is not part of Ember

fn main():
    pass
