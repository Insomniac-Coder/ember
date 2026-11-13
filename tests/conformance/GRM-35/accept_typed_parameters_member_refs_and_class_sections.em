#$ test: parse-pass
#$ rules: GRM-35, FFI-12, FFI-40a, LEX-15
# `[GRM-35]` — in an entry that is not a macro a parameter may carry, before its contract words, the
# word and the type the header gives it (`owns tex: Texture`, `value: c_int`, `s: *c_char`); a `rename`
# or a `hide` with a parameter list names one member of a name (`rename put(v: c_long) as put_long`,
# `hide put(v: c_long)`, `rename get() -> f64 as get_f64`); an `overlay_class` section holds the entries
# of a C++ class's members (its methods, constructors `new`, static functions), `@ffi(const)` above a
# method that only reads its object; an `unsafe fn` entry states what its callers must make true; a
# result is a type then contract words, or contract words alone.

unsafe overlay cpp "engine/Widget.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn widget_draw(owns tex: Texture, value: c_int, s: *c_char, n: count(value),) -> c_int nonnull
    fn widget_size(w: borrowed) -> nonnull borrowed
    unsafe fn widget_poke(p: *c_char) -> c_int
    rename put(v: c_long) as put_long
    hide put(v: c_long)
    rename get() -> f64 as get_f64
    rename plain as renamed_plain
    hide hidden_one

    class Widget:
        new(width: c_int, height: c_int)
        @ffi(const)
        fn width() -> c_int
        fn resize(w: c_int, h: c_int)
        fn make(kind: c_int)
        rename width() as get_width
        hide internal_reset

fn main():
    println(1)
