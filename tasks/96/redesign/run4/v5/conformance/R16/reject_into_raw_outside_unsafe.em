#$ test: compile-fail
#$ rules: UNS-1, UNS-12, FFI-54
## design rules: R16
## R16 (verdicts 91 and 58's corrected record): for a C function whose declaration cannot say it keeps
## the object (a plain pointer, `fn set_user_data(p: *mut Widget)`), "`second.into_raw()` gives the
## object's address and stops Ember freeing it ...; the compiler makes that step itself ... Outside
## `unsafe:` it is refused (E3100 ...)" (section 3's E3100 message, with the lines to write). The nearest
## allowed program is `accept_into_raw_inside_unsafe_gives_c_the_address.em`.

unsafe extern "C":
    @ffi(destroys(widget_free))
    type Widget
    fn widget_free(widget: *mut Widget)
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> owns Widget
    fn set_user_data(p: *mut Widget)

fn address_for_c(owns w: owns Widget) -> *mut Widget:
    return w.into_raw()    #$ error[E3100]: `into_raw()` stops Ember from freeing the widget, so a mistake leaks it or frees it twice; write it inside `unsafe:`: above line 18 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 18 under it

fn main():
    second = widget_new(2)
    p = address_for_c(second)
    # SAFETY: C keeps the widget as its user data and frees it with `widget_free`
    unsafe:
        set_user_data(p)
