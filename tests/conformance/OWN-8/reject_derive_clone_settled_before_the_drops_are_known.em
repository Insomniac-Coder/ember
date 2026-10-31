#$ test: compile-fail
#$ rules: OWN-8, STR-5
# `[OWN-8]` — a derived `Clone` needs every field to implement it, however
# early another type's instance is made. The extension below makes `Tag[i32]`
# while types are still being collected, before `Resource`'s `drop` is known;
# settling `Bag`'s request then accepted it (D-336).

struct Resource:
    id: i32

    fn drop():
        pass

@derive(Clone)
struct Bag:
    item: Resource #$ error[E2040]: field `item` has type `Resource`, which does not implement `Clone`

interface Weigh[W]:
    fn weigh(w: W) -> i32

@derive(Clone)
[T]
struct Tag:
    v: T

extend i32 implements Weigh[Tag[i32]]:
    fn weigh(w: Tag[i32]) -> i32:
        return this + w.v

fn main():
    _bag = Bag(Resource(1))
