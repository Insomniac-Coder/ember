# Clause BCK-4.2: "a view, or a value holding views, kept or returned where it would outlive what it
# points into, a value made in a memory manager's instance included, is `E3063`."
#$ test: compile-fail
#$ rules: BCK-4, TYP-15, LT-45, LT-47, LT-4
# The spec gives no message text for these cases: codes only.

struct Mesh:
    points: Array[float]

fn view_of_a_local() -> StrView:
    s = String("abc")
    return s[0..1]    #$ error[E3063]

fn views_of_a_local() -> Array[StrView]:
    s = String("ab")
    ws = [s[0..1], s[1..2]]
    return ws    #$ error[E3063]

fn put_a_local(ref mut out: StrView):
    s = String("abc")
    out = s[0..1]    #$ error[E3063]

fn made_in_a_local_arena() -> owns Mesh using Arena:
    level = Arena.with_capacity(1024)
    m: owns Mesh using Arena = owns Mesh(points=[1.0]) using level
    return m    #$ error[E3063]

fn main():
    println(view_of_a_local(), views_of_a_local().len())
    name: StrView = "x"
    put_a_local(ref mut name)
    println(made_in_a_local_arena().points.len())
