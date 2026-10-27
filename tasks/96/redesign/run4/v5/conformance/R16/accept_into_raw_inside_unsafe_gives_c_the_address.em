#$ test: compile-pass
#$ rules: TIER-1
## design rules: R16
## R16 (verdict 91): the lines E3100 gives: `into_raw()` inside `unsafe:`, with a `# SAFETY:` note saying
## why it is sound; the widget's name ends with `address_for_c`'s `owns w`, and Ember frees nothing.

unsafe extern "C":
    @ffi(destroys(widget_free))
    type Widget
    fn widget_free(widget: *mut Widget)
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> owns Widget
    fn set_user_data(p: *mut Widget)

fn address_for_c(owns w: owns Widget) -> *mut Widget:
    # SAFETY: the address goes to `set_user_data`, whose C side keeps the widget and frees it with `widget_free`
    unsafe:
        return w.into_raw()

fn main():
    second = widget_new(2)
    p = address_for_c(second)
    # SAFETY: C keeps the widget as its user data and frees it with `widget_free`
    unsafe:
        set_user_data(p)
