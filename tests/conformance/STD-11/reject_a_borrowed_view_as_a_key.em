#$ test: compile-fail
#$ rules: STD-11, STD-16, TYP-15
# `[STD-11]`, `[TYP-15]` (ODR-069, SP-013) — a `Map`'s or `Set`'s storage has
# no bounding region, so a view that borrows a local may not go in: the
# error is at the call that would store it.

fn main():
    labels: Map[str, int] = {"start": 1}
    colours: Set[str] = Set[str]()
    s = String.from("zed")
    labels.insert(s.as_str(), 2) #$ error[E3063]: `str` is a view, so it may not be passed to `insert`
    labels[s.as_str()] = 3 #$ error[E3063]: `str` is a view, so it may not be passed to `index_set`
    colours.add(s.as_str()) #$ error[E3063]: `str` is a view, so it may not be passed to `add`
    println(labels, colours)
