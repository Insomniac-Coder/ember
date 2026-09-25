#$ test: compile-fail
#$ rules: STD-11, TYP-15
# ODR-036 — a `Map`'s keys and values and a `Set`'s elements are not views:
# a view inside would make the map itself a view, confined to locals.

fn main():
    m = Map[str, int]()  #$ error[E3063]: `str` is a view, so it may not be a `Map` key or value
    s = Set[str]()  #$ error[E3063]: `str` is a view, so it may not be a `Set` element
