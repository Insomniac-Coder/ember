#$ test: compile-fail
#$ rules: WK-11, TYP-45
# `[WK-11]` — a value held `owns` lives in a block with no header and is not a target: a `weakref`
# of it is `E2119` (the rule's own example).

struct Mesh:
    points: Array[float]

fn main():
    b: owns Mesh = owns Mesh(points=[0.0])
    w = weakref b                     #$ error[E2119]: `b` holds its mesh in a block with no header, which a link cannot test; a link names a class object or a value held `shares` or `thread_shared`: write `b: shares Mesh = shares Mesh(points=[0.0])` in place of line 10, then `w = weakref b`
