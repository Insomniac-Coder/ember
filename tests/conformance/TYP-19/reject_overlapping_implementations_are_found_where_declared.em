#$ test: compile-fail
#$ rules: TYP-19, TYP-20, DIA-14
# D-396 (ODR-092) — two implementations of one interface whose types could be
# one type are `E2041` where they are declared, though no instance meets both,
# and the error names both. A bound on a type left open never separates two:
# a type that is both `Copy` and `Clone` can always be declared. Each was
# accepted before unless a program used an instance both applied to, and
# then reported there, at a use, once per instance.

interface First:
    fn first(self) -> int

interface Second:
    fn second(self) -> int

interface Third:
    fn third(self) -> int

interface Fourth:
    fn fourth(self) -> int

interface Fifth:
    fn fifth(self) -> int

struct Wrap[T]:
    x: T

struct Pair[A, B]:
    a: A
    b: B

struct Box[T] implements Fourth:
    x: T

    fn fourth(self) -> int:
        return 1

# Two parameters of different names, bounded apart: `Wrap[int]` is both.
extend[T: Copy] Wrap[T] implements First:
    fn first(self) -> int:
        return 1

extend[U: Clone] Wrap[U] implements First:     #$ error[E2041]: `Wrap[U]` and `Wrap[T]` can be one type, and both implement `First`
    fn first(self) -> int:
        return 2

# Neither is an instance of the other, and `Pair[str, int]` is both.
extend[T] Pair[T, int] implements Second:
    fn second(self) -> int:
        return 1

extend[U] Pair[str, U] implements Second:      #$ error[E2041]: the other implementation is here
    fn second(self) -> int:
        return 2

# A nested instance.
extend[T] Wrap[T] implements Third:
    fn third(self) -> int:
        return 1

extend[T] Wrap[Wrap[T]] implements Third:      #$ error[E2041]: `Wrap[Wrap[T]]` and `Wrap[T]` can be one type
    fn third(self) -> int:
        return 2

# A type's own `implements` and an extension of it.
extend[U] Box[U] implements Fourth:            #$ error[E2041]: `Box[U]` and `Box[T]` can be one type, and both implement `Fourth`
    fn fourth(self) -> int:
        return 2

# A generic implementation and a concrete one no program uses.
extend[T] Pair[T, T] implements Fifth:
    fn fifth(self) -> int:
        return 1

extend Pair[bool, bool] implements Fifth:      #$ error[E2041]: `Pair[bool, bool]` and `Pair[T, T]` can be one type
    fn fifth(self) -> int:
        return 2

fn main():
    println("never built")
