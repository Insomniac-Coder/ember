#$ test: compile-fail
#$ rules: TYP-16
# A default fills an omitted trailing type argument; it adds no room for more.

[K, H = int]
struct Holder:
    k: K

fn main():
    h = Holder[int, int, int](1)  #$ error[E2020]: `Holder` takes 2 type arguments, found 3
