#$ test: compile-pass
#$ rules: TIER-1, FN-1
## design rules: R16, R4
## R16 (verdict 91): "a C function declared as keeping an object (`@ffi(param(widget, owns, one))` with
## `owns widget: owns Widget`) is given one by an ordinary hand-over, `widget_take(first)`: the name ends
## and Ember frees nothing, with no word" (R4: an argument for an `owns` parameter is handed over with no
## word); the widget C may keep is an `owns Widget` (R3).

unsafe extern "C":
    @ffi(destroys(widget_free))
    type Widget
    fn widget_free(widget: *mut Widget)
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> owns Widget
    @ffi(param(widget, owns, one))
    safe fn widget_take(owns widget: owns Widget)

fn main():
    first = widget_new(1)
    widget_take(first)
