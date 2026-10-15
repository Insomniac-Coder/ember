#$ test: run-pass
#$ rules: PHIL-9, TYP-11
#$ profiles: debug, release
#$ stdout: 3 3
#$ assert-c: contains(struct em_Point {)
#$ assert-c-order: "struct em_Point {" then "    int32_t x;"
#$ assert-c-order: "    int32_t x;" then "    int32_t y;"
#$ assert-c-order: "struct em_obj_Counter {" then "    ember_obj_header _header;"
# `[PHIL-9]` — a class instance carries a header, which is what its dynamic checks use; a plain
# struct has no hidden state: its C is exactly its fields, in order, with C layout (`[TYP-11]`).

struct Point:
    x: i32
    y: i32

class Counter:
    n: i32

fn main():
    p = Point(1, 2)
    c = Counter(3)
    println(p.x + p.y, c.n)
