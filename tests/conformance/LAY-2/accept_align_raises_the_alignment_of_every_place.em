#$ test: run-pass
#$ rules: LAY-2, HEAP-9
#$ profiles: debug, development, release
#$ assert-c: contains("EMBER_ALIGNED(64)")
#$ assert-c: contains("ember_vec_push_aligned")
#$ stdout: 64 64 128 64 | 128 32 16 8 16
#$ 0 0 0 0 0 0 0 0 0 0
#$ 0 0 0 0 0
#$ 217 3
#$ 32 32 64
# `[LAY-2]` — `@align(N)` raises a type's alignment to `N` and makes its size a multiple of `N`
# (a struct's or an enum's), and every place holding it honours it: a local, a field of another
# struct, an element of a tuple, a `Box`'s payload and, by `[HEAP-9]`, every buffer of an `Array`
# — from its first push through growth, `reserve`, a literal, `insert`, `extend`, `drain`, `clone`
# and a comprehension. An alignment the type already has changes nothing.

@align(64)
@derive(Copy)
struct Line:
    a: u8
    b: u64

struct Holder:
    tag: u8
    line: Line

@align(32)
enum Slot:
    Empty
    Full(u64)

@align(8)
struct Wide:
    a: u64
    b: u8

@align(2)
struct Twice:
    a: u32

@align(32)
struct Cell32[T]:
    value: T

fn off(p: *Line) -> int:
    unsafe:  # SAFETY: only the address is read, as a number; nothing is accessed through it
        return ((p as usize) % 64) as int

fn listed(xs: Span[Line]) -> int:
    return off(xs.as_ptr())

fn main():
    println(size_of[Line](), align_of[Line](), size_of[Holder](), offset_of[Holder](line), "|", size_of[(u8, Line)](), size_of[Slot](), size_of[Wide](), align_of[Wide](), size_of[Twice]() * 4)
    xs: Array[Line] = Array[Line]()
    xs.push(Line(a = 1, b = 2))
    first = listed(xs.as_span())
    for i in 0..100:
        xs.push(Line(a = 1, b = i as u64))
    grown = listed(xs.as_span())
    reserved: Array[Line] = Array[Line]()
    reserved.reserve(10)
    reserved.push(Line(a = 9, b = 9))
    literal = [Line(a = 1, b = 1), Line(a = 2, b = 2)]
    literal.insert(1, Line(a = 3, b = 3))
    literal.extend(xs.as_span())
    drained = literal.drain(0..2)
    copy = xs.clone()
    built = [Line(a = 1, b = i as u64) for i in 0..20]
    println(first, grown, listed(reserved.as_span()), listed(literal.as_span()), listed(drained.as_span()), listed(copy.as_span()), listed(built.as_span()), len(xs) - 101, len(literal) - 102, len(drained) - 2)
    local = Line(a = 5, b = 6)
    h = Holder(tag = 1, line = local)
    pair = (7 as u8, local)
    boxed = Box(Line(a = 1, b = 100))
    full = Slot.Full(5)
    println(off(ref_to_ptr(ref local)), off(ref_to_ptr(ref h.line)), off(ref_to_ptr(ref pair.1)), off(ref_to_ptr(boxed.get())), 0)
    held = match full:
        Slot.Empty => 0 as u64
        Slot.Full(n) => n
    println(h.line.b + boxed.b + pair.1.b + held + 100 as u64, built[3].b)
    println(size_of[Cell32[u8]](), align_of[Cell32[u64]](), size_of[Cell32[Line]]())
