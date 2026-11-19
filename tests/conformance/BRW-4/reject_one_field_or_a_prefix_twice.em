# Clause BRW-4.1: "`ref mut a.x` and `ref mut a.y` may be in use together when `x` and `y` are
# different fields" -- and not the same field, or a field and a path that extends it, which "may be
# one" (`[BCK-3]`: "when one is a prefix or an extension of the other").
#$ test: compile-fail
#$ rules: BRW-4, BRW-1, BCK-3
# Codes only.

struct Inner:
    a: int
    b: int

struct Outer:
    inner: Inner
    tag: int

class Ship:
    pos: Inner
    fuel: int = 0

fn the_same_field():
    o = Outer(inner=Inner(a=1, b=2), tag=3)
    x = ref mut o.tag
    y = ref mut o.tag    #$ error[E3022]
    x += 1
    y += 1

fn a_prefix_and_its_extension():
    o = Outer(inner=Inner(a=1, b=2), tag=3)
    whole = ref mut o.inner
    part = ref mut o.inner.a    #$ error[E3022]
    whole.b += 1
    part += 1

fn a_class_objects_prefix():
    s = Ship(pos=Inner(a=0, b=0))
    whole = ref mut s.pos
    part = ref mut s.pos.b    #$ error[E3022]
    whole.a += 1
    part += 1

fn main():
    the_same_field()
    a_prefix_and_its_extension()
    a_class_objects_prefix()
