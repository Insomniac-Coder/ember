#$ test: compile-fail
#$ rules: STD-9, LEX-19, TXT-9, TYP-17, IFC-4, TYP-39
# Every formatting entry point needs a text capability from a generic value.
# Nested values name a missing bound rather than an unimplemented feature.
# A format spec must not produce a second error after the missing bound.

interface Holder:
    type Item

struct Wrapper[T]:
    item: T

fn direct_fstring[T](x: T) -> String:
    return f"{x}"    #$ error[E2040]

fn direct_debug_fstring[T](x: T) -> String:
    return f"{x!r}"    #$ error[E2040]

fn direct_spec[T](x: T) -> String:
    return f"{x:>8}"    #$ error[E2040]

fn direct_array[T](xs: Array[T]):
    println(xs)    #$ error[E2040]

fn direct_option[T](x: Option[T]):
    println(x)    #$ error[E2040]

fn direct_tuple[T](x: (int, T)):
    println(x)    #$ error[E2040]

fn direct_wrapper[T](x: Wrapper[T]):
    println(x)    #$ error[E2040]

fn direct_to_string[T](x: T) -> String:
    return x.to_string()    #$ error[E2040]

fn projected_fstring[H: Holder](x: H.Item) -> String:
    return f"{x}"    #$ error[E2040]

fn projected_spec[H: Holder](x: H.Item) -> String:
    return f"{x:>8}"    #$ error[E2040]

fn projected_array[H: Holder](xs: Array[H.Item]):
    println(xs)    #$ error[E2040]

fn projected_nested[H: Holder](x: Option[Array[H.Item]]):
    println(x)    #$ error[E2040]

fn projected_to_string[H: Holder](x: H.Item) -> String:
    return x.to_string()    #$ error[E2040]

fn transparent_box[T](x: Box[T]):
    println(x)    #$ error[E2040]: requires `Display` or `Debug`

fn transparent_cell[T: Copy](x: Cell[T]):
    println(x)    #$ error[E2040]: requires `Display` or `Debug`

fn main():
    pass
