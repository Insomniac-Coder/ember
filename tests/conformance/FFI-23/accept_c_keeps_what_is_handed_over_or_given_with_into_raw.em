#$ test: run-pass
#$ rules: FFI-23, FFI-54, FFI-11, OWN-12
#$ profiles: debug, development, release
#$ stdout: 1 0
#$ stdout: kept 2
#$ stdout: 1 1
# `[FFI-23]` — `Retained` gives no object away and keeps none for C: an Ember value C keeps is
# handed to a parameter whose contract says `owns` (the name ends and Ember frees nothing,
# `[FFI-11]`), or, for a C function whose declaration cannot say so, given with `into_raw()`
# inside `unsafe:` (`[FFI-54]`); `[FFI-54]`'s example. The C side is the program's own C
# functions, so the test links on its own: `widget_take` keeps what it is given, and so does
# `set_user_data`; each counts.

from std.sync import Atomic

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @ffi(link_name="kept_widget_free")
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)
    @ffi(result(returns_owned(destructor=widget_free), one), link_name="kept_widget_new")
    safe fn widget_new(id: i32) -> owns Widget
    @ffi(param(widget, owns, one), link_name="kept_widget_take")
    safe fn widget_take(owns widget: owns Widget)
    @ffi(link_name="kept_set_user_data")
    @safety("`p` is a widget whose ownership C takes")
    unsafe fn set_user_data(p: *mut Widget)

static TAKEN: Atomic[i64] = Atomic(0)
static KEPT: Atomic[i64] = Atomic(0)

pub extern "C" fn kept_widget_new(id: i32) -> *mut Widget:
    # SAFETY: an address no Ember value has, which only these functions see
    unsafe:
        return (4096 * id as usize) as *mut Widget

pub extern "C" fn kept_widget_free(widget: *mut Widget):
    println("freed")

pub extern "C" fn kept_widget_take(widget: *mut Widget):
    TAKEN.fetch_add(1)

pub extern "C" fn kept_set_user_data(p: *mut Widget):
    KEPT.fetch_add(1)
    # SAFETY: the address `kept_widget_new` made for the widget with id 2
    unsafe:
        println("kept", p as usize // 4096)

fn address_for_c(owns w: owns Widget) -> *mut Widget:
    # SAFETY: the address goes to `set_user_data`, whose C side keeps the widget and frees it with `widget_free`
    unsafe:
        return w.into_raw()

fn main():
    first = widget_new(1)
    widget_take(owns first)
    println(TAKEN.load(), KEPT.load())
    second = widget_new(2)
    p = address_for_c(owns second)
    # SAFETY: C keeps the widget as its user data and frees it with `widget_free`
    unsafe:
        set_user_data(p)
    println(TAKEN.load(), KEPT.load())
