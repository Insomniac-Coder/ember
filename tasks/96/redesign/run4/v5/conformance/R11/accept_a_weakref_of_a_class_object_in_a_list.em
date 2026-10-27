#$ test: run-pass
#$ rules: WK-16
#$ stdout: 1
## design rules: R11, R8
## R11: the nearest allowed program to `reject_a_weakref_of_a_struct_stored_inline_in_a_list.em`: a
## `Pos` object lives in its own block, so a `weakref` link to the one in `points[0]` is allowed, and
## opening it tests that block's header (section 2, "one test of the target's header").

class Pos:
    x: int = 0

class Marker:
    at: weakref Pos = None

fn main():
    points = [Pos(x=1), Pos(x=2)]
    m = Marker()
    m.at = weakref points[0]
    if m.at exists as p:
        println(p.x)
