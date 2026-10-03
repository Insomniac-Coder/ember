#$ test: run-pass
#$ rules: TYP-13, TYP-14, TYP-15, SPN-1, SPN-3, ERR-4, OWN-6, GRM-15
#$ profiles: debug, release, shipping
#$ stdout: 2 6 10
#$ stdout: none
#$ stdout: 6 10
#$ stdout: 0
# Genuine dereference/field storage holds a bounded exclusive view. None
# writes the descriptor fields; Some construction, take(), and outside-arm
# Option moves lead to owned payload extraction. Unlike Span, MutSpan is
# moved. No plain local is declared inside an owned match arm, avoiding the
# independent end-of-arm hoisting read of an already consumed local.

@view
struct MutableSlot:
    value: Option[MutSpan[int]]

@noinline
fn mutable_span_chain():
    values = [5, 7]
    slot = MutableSlot(None)
    slot_ref: ref mut MutableSlot = ref mut slot
    slot_ref.value = None
    slot_ref.value = Some(values.as_mut_span())
    taken: Option[MutSpan[int]] = None
    taken = slot_ref.value.take()
    moved_option = taken
    forwarded = moved_option
    match owned forwarded:
        Some(view):
            view[0] = view[0] + 1
            view[1] = view[0] + 4
            println(view.len(), view[0], view[1])
        None:
            panic("taken mutable span lost its tag")
    match slot_ref.value:
        Some(_):
            panic("take did not leave None")
        None:
            println("none")
    println(values[0], values[1])
    empty_values: Array[int] = []
    empty: Option[MutSpan[int]] = None
    empty = Some(empty_values.as_mut_span())
    match owned empty:
        Some(view):
            println(view.len())
        None:
            panic("empty mutable span became None")

fn main():
    mutable_span_chain()
