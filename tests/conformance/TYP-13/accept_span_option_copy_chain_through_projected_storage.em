#$ test: run-pass
#$ rules: TYP-13, TYP-14, TYP-15, SPN-1, SPN-2, LT-20
#$ profiles: debug, release, shipping
#$ stdout: 20 30 2
#$ stdout: 3 10
#$ stdout: 0
#$ stdout: none
#$ stdout: 99
# Local fixed arrays can hold bounded views. Dereference and constant-index
# projections remain real descriptor storage. Element 0 has field producers;
# element 1 is written whole and deliberately has no later field producer.

@noinline
fn shared_span_chain():
    values = [10, 20, 30]
    slots: [Option[Span[int]]; 2] = [None, None]
    slot_ref: ref mut [Option[Span[int]]; 2] = ref mut slots
    whole = values.as_span()
    slot_ref[0] = None
    cut = whole[1..3]
    slot_ref[0] = Some(cut)
    slot_ref[1] = Some(whole)
    copied = slot_ref[0]
    again = copied
    match again:
        Some(view):
            _alias = view
            _wrapped = Some(_alias)
            match _wrapped:
                Some(result):
                    println(result[0], result[1], result.len())
                None:
                    panic("copied shared span lost its tag")
        None:
            panic("projected shared span lost its tag")
    plain = slot_ref[1]
    match plain:
        Some(view):
            println(view.len(), view[0])
        None:
            panic("whole shared span lost its tag")
    empty: Option[Span[int]] = None
    empty = Some(whole[1..1])
    match empty:
        Some(view):
            println(view.len())
        None:
            panic("empty shared span became None")
    slot_ref[0] = None
    match slot_ref[0]:
        Some(_):
            panic("cleared projected span stayed Some")
        None:
            println("none")
    values[0] = 99
    println(values[0])

fn main():
    shared_span_chain()
