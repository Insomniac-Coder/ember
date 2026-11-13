# Clause BCK-4.2: "a view, or a value holding views, kept or returned where it would outlive what it
# points into, a value made in a memory manager's instance included, is `E3063`."
#$ test: compile-fail
#$ rules: BCK-4, LT-4, ALC-1
# The spec gives no message text for this case: the code only.

struct Mesh:
    points: Array[float]

fn made_in_a_local_arena() -> owns Mesh using Arena:
    level = Arena.with_capacity(1024)
    m: owns Mesh using Arena = owns Mesh(points=[1.0]) using level
    return m    #$ error[E3063]

fn main():
    println(made_in_a_local_arena().points.len())
