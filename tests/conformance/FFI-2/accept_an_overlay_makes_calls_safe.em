#$ test: run-pass
#$ rules: FFI-2, FFI-2a, FFI-13, FFI-30a
#$ profiles: debug, development, release
#$ stdout: 7 10 3 3 3
#$ stdout: positive 5 9 12
#$ stdout: 6
#$ stdout: 1
# `[FFI-2]` — an overlay supplies the facts the header cannot state, and each function it has an
# entry for is safe to call: no `unsafe`. `widget_id`, with no entry, stays unsafe. `[FFI-2a]` —
# a pointer's mutability is derived from `const`, never written: `const int*` with
# `borrowed count(n)` is a `Span[i32]`, `int*` a `MutSpan[i32]`, `int*` with `borrowed one` a
# `mut` parameter. `[FFI-13]` — the overlay's `extend` block adds a method to the header's struct.
# `[FFI-30a]` — it renames a function (`add`) and hides one (`widget_free`), and changes no type:
# `Size` is the header's struct.

import c "./widgets.c" with (overlay = "widgets_overlay.em") as w

# The box frees its widget, with `widget_free`, when it is dropped here.
fn id_of_new(n: i32) -> i32:
    widget = w.widget_new(n)
    id: i32 = 0
    unsafe:  # SAFETY(ffi): the box keeps the widget alive for the call
        id = w.widget_id(widget.as_ptr() as *w.Widget)
    return id

fn main():
    id = id_of_new(7)
    items: Array[i32] = [1, 2, 7]
    println(id, w.widget_sum(items.as_span()), items.len(), 3, 3)
    w.widget_fill(items.as_mut_span(), 3)
    value: i32 = 6
    w.widget_double(value)
    println(w.widget_label(1).to_str().unwrap(), w.add(2, 3), w.widget_sum(items.as_span()), value)
    size = w.Size(w=2, h=3)
    println(size.area())
    println(w.widget_freed())
