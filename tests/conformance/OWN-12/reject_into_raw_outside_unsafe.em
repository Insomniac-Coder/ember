#$ test: compile-fail
#$ rules: OWN-12, FFI-54, UNS-1
# `[OWN-12]`, `[FFI-54]`, `[UNS-1]` — `into_raw()` outside `unsafe:` is `E3100`, whose message says
# what a mistake costs and gives the block to write.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)

fn address_for_c(owns w: owns Widget) -> *mut Widget:
    return w.into_raw()    #$ error[E3100]: `into_raw()` stops Ember from freeing the widget, so a mistake leaks it or frees it twice; write it inside `unsafe:`: above line 13 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 13 under it

fn main():
    pass
