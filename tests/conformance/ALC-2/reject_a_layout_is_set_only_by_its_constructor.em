#$ test: compile-fail
#$ rules: ALC-2, MOD-2
## `[ALC-2]` — `Layout`'s fields are `pub(read)`: outside `std.mem` a field is never written, so the
## only `Layout` there is is the one its constructor checked.

from std.mem import Layout

fn main():
    l = Layout(16, 8)
    l.align = 3    #$ error[E1050]: `std.mem.Layout.align` is read-only outside its module
    println(l.align)
