# Clause BCK-4.2: "a view, or a value holding views, kept or returned where it would outlive what it
# points into, a value made in a memory manager's instance included, is `E3063`."
#$ test: compile-fail
#$ rules: BCK-4, TYP-15, LT-45, LT-47
# The spec gives no message text for these cases: codes only. The value made in an instance is
# reject_a_value_made_in_an_instance_returned_past_it (it also needs `using`, M10).

fn view_of_a_local() -> StrView:
    s = String("abc")
    return s[0..1]    #$ error[E3063]

fn views_of_a_local() -> Array[StrView]:
    s = String("ab")
    ws = [s[0..1], s[1..2]]
    return ws    #$ error[E3063]

fn put_a_local(ref mut out: StrView):
    s = String("abc")
    out = s[0..1]    #$ error[E3063]

fn main():
    println(view_of_a_local(), views_of_a_local().len())
    name: StrView = "x"
    put_a_local(ref mut name)
