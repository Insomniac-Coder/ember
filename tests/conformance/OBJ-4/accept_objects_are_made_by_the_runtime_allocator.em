#$ test: run-pass
#$ rules: OBJ-4, RT-1, OBJ-1
#$ profiles: debug, release, shipping
#$ assert-c: contains("ember_obj_new(&em_ti_Point)")
#$ assert-c: !contains("malloc(")
#$ assert-c: !contains("calloc(")
#$ stdout: 3 4
# `[OBJ-4]` — a class object is made by the runtime's object allocator
# (`ember_obj_new`, which takes its memory from `[RT-1]`'s allocator and sets
# the header), never by a C allocation in the program's own code.

class Point:
    x: int
    y: int

fn main():
    p = Point(3, 4)
    println(p.x, p.y)
