#$ test: compile-fail
#$ rules: OWN-8, STR-5

# A derived `Clone` needs every field to implement it. `Resource` has a
# `drop` and no `clone`, so it does not (an `Array` or `String` does).

struct Resource:
    id: i32

    fn drop(mut self):
        pass

@derive(Clone)
struct Bag:
    item: Resource #$ error[E2040]: field `item` has type `Resource`, which does not implement `Clone`

fn main():
    _bag = Bag(Resource(1))
