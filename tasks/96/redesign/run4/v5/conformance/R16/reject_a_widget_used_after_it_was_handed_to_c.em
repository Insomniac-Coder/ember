#$ test: compile-fail
#$ rules: OWN-3
## design rules: R16, R4, R1
## R16 (verdict 91): `widget_take(first)` hands the widget to C: "the name ends and Ember frees nothing";
## a later use of `first` is E3040 (R1, R4), so C is never given one widget twice (which would free it
## twice). The nearest allowed program is
## `accept_a_c_function_declared_owned_takes_the_widget_with_no_word.em`.

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
    widget_take(first)    #$ error[E3040]: `first` was handed over to `widget_take` on line 19 and is used on line 20; C keeps that widget now, so give it another: write `widget_take(widget_new(2))` in place of line 20
