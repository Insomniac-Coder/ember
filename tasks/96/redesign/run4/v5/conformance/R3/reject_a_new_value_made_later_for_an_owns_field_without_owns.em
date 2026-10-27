#$ test: compile-fail
#$ rules: OWN-10, TYP-46
#$ help: write `e.mesh = owns Mesh(verts=[1, 2])`
## design rules: R3
## R3: a value made later for an `owns` place says `owns` too (verdict 97): `e.mesh = Mesh(...)` makes a
## new mesh for a field that owns its mesh alone, so the line says so, as the line that first made one
## does (E2123, `V5/rules.md` section 3's message in this program's names). The nearest programs that
## are allowed are `run_a_new_value_made_later_for_an_owns_field_says_owns_and_ends_the_old_one` and,
## for a mesh that already has a name, `accept_an_existing_mesh_handed_to_an_owns_field_with_no_word`.

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: owns Mesh

fn main():
    e = Entity(mesh=owns Mesh(verts=[1]))
    e.mesh = Mesh(verts=[1, 2])            #$ error[E2123]: `mesh` owns its mesh; say so where the new one is made
    println(e.mesh.verts.len())
