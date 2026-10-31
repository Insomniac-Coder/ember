#$ test: compile-fail
#$ rules: STD-9, LEX-19, TXT-9, TYP-17, IFC-4, TYP-39
# Every formatting entry point needs a text capability from a generic value.
# Nested values name a missing bound rather than an unimplemented feature.
# A format spec must not produce a second error after the missing bound.

interface Holder:
    type Item

[T]
struct Wrapper:
    item: T

[T]
fn direct_fstring(x: T) -> String:
    return f"{x}"    #$ error[E2040]

[T]
fn direct_debug_fstring(x: T) -> String:
    return f"{x!r}"    #$ error[E2040]

[T]
fn direct_spec(x: T) -> String:
    return f"{x:>8}"    #$ error[E2040]

[T]
fn direct_array(xs: Array[T]):
    println(xs)    #$ error[E2040]

[T]
fn direct_option(x: Option[T]):
    println(x)    #$ error[E2040]

[T]
fn direct_tuple(x: (int, T)):
    println(x)    #$ error[E2040]

[T]
fn direct_wrapper(x: Wrapper[T]):
    println(x)    #$ error[E2040]

[T]
fn direct_to_string(x: T) -> String:
    return x.to_string()    #$ error[E2040]

[H: needs Holder]
fn projected_fstring(x: H.Item) -> String:
    return f"{x}"    #$ error[E2040]

[H: needs Holder]
fn projected_spec(x: H.Item) -> String:
    return f"{x:>8}"    #$ error[E2040]

[H: needs Holder]
fn projected_array(xs: Array[H.Item]):
    println(xs)    #$ error[E2040]

[H: needs Holder]
fn projected_nested(x: Option[Array[H.Item]]):
    println(x)    #$ error[E2040]

[H: needs Holder]
fn projected_to_string(x: H.Item) -> String:
    return x.to_string()    #$ error[E2040]

[T]
fn transparent_box(x: Box[T]):
    println(x)    #$ error[E2040]: requires `Display` or `Debug`

[T: needs Copy]
fn transparent_cell(x: Cell[T]):
    println(x)    #$ error[E2040]: requires `Display` or `Debug`

fn main():
    pass
