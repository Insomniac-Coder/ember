#$ test: run-pass
#$ rules: SPN-1, SPN-2, BRW-2, EXP-1, TYP-13, LT-20, CG-C-3
#$ profiles: debug, release, shipping
#$ stdout: true true
#$ stdout: Aé🌶 7
#$ stdout: Aé🌶 7
#$ stdout: 1 élan 5
#$ stdout: some-empty
#$ stdout: some-text 7
#$ stdout: none
#$ stdout: 3 65 0 66
# Source-only fixture. Root must parse and validate it before adoption.
# No filename, benchmark, literal-content, or loop-shape optimization gate.

@noinline
fn borrow_explicit(text: String) -> str:
    return text.as_str()

@noinline
fn borrow_implicit(text: String) -> str:
    return text

@noinline
fn descriptor_chain(text: String, keep: bool) -> Option[str]:
    if not keep:
        return None
    first = text.as_str()
    copied = first
    slots: [Option[str]; 2] = [None, None]
    slot_ref: ref mut [Option[str]; 2] = ref mut slots
    slot_ref[1] = Some(copied)
    through_projection = slot_ref[1]
    again = through_projection
    match again:
        Some(result):
            return Some(result)
        None:
            return None

@noinline
fn next_index(mut reads: int) -> int:
    reads += 1
    return 0

fn main():
    empty = String()
    reserved = String.with_capacity(32)
    println(borrow_explicit(empty).is_empty(), borrow_implicit(reserved).is_empty())
    text = String.from("Aé🌶")
    a = borrow_explicit(text)
    println(a, a.len())
    b = borrow_implicit(text)
    println(b, b.len())
    owners: Array[String] = ["élan"]
    reads = 0
    selected = owners[next_index(reads)].as_str()
    println(reads, selected, selected.len())
    match descriptor_chain(empty, true):
        Some(view):
            if view.is_empty():
                println("some-empty")
            else:
                panic("empty view lost its length")
        None:
            panic("Some(empty) became None")
    match descriptor_chain(text, true):
        Some(view):
            println("some-text", view.len())
        None:
            panic("text view became None")
    match descriptor_chain(text, false):
        Some(_):
            panic("None became Some")
        None:
            println("none")
    bytes: Array[u8] = [65, 0, 66]
    span = bytes.as_span()
    println(span.len(), span[0], span[1], span[2])
