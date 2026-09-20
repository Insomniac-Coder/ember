#$ test: compile-fail
#$ rules: OWN-8

@derive(Clone)
struct Bag:
    values: Array[i32] #$ error[E2040]: field `values` has type `Array[i32]`, which does not implement `Clone`

fn main():
    _bag = Bag(Array[i32]())
