#$ test: run-pass
#$ rules: FFI-33b, THR-1, STA-1
#$ profiles: debug, development, release
#$ stdout: 1
# `[FFI-33b]` — `ForeignBox` is gone: an `owns T` of a foreign type with no thread contract is `Send`
# (D-834), so a `Mutex[Array[owns Widget]]` is `Sync` and is a `static` (`[STA-1]`). The destructor
# is the program's own C function, so the test links on its own.

from std.ffi import adopt
from std.sync import Mutex

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @ffi(link_name="held_widget_new")
    @safety("C gives a fresh widget, which the caller owns")
    unsafe fn widget_new() -> *mut Widget
    @ffi(link_name="held_widget_free")
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)

pub extern "C" fn held_widget_new() -> *mut Widget:
    # SAFETY: an address no Ember value has, which only `held_widget_free` sees
    unsafe:
        return 4096 as *mut Widget

pub extern "C" fn held_widget_free(widget: *mut Widget):
    pass

static WIDGETS: Mutex[Array[owns Widget]] = Mutex([])

fn made() -> owns Widget:
    # SAFETY: a fresh widget, owned only by this `owns Widget`
    unsafe:
        return adopt(widget_new())

fn keep():
    with all = WIDGETS.lock():
        all.push(owns made())

fn main():
    keep()
    with all = WIDGETS.lock():
        println(all.len())
