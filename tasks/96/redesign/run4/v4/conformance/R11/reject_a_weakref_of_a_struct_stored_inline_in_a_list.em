#$ test: compile-fail
#$ rules: R11
## R11: "its target lives on its own (a class object, or a value held `shares` or `thread_shared`), so
## a `weakref` of a part, a field or an element stored inline, is refused (E2119)": a `Pos` is stored
## inside the list's buffer, so there is no header a link could test once it is gone.

struct Pos:
    x: int = 0

class Marker:
    at: weakref Pos = None

fn main():
    points = [Pos(x=1), Pos(x=2)]
    m = Marker()
    m.at = weakref points[0]            #$ error[E2119]: `points[0]` is a `Pos` stored inside the list `points`, and a `weakref` link needs an object that lives on its own; make `Pos` a class, whose objects each live in their own block: write `class Pos:` in place of line 7
    if m.at exists as p:
        println(p.x)
