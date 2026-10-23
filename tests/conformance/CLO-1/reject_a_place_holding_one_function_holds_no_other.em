#$ test: compile-fail
#$ rules: CLO-1, FN-6, DIA-2
# ODR-278, `[CLO-1]` — each function has a type of its own, so a place declared from one holds
# that one: assigned another inside a local function, which the body's own assignments do not
# show, it is `E2020`, and the help names the shape that holds any function of it. (Assigned
# again in the body itself, the place is the shape: `FN-6/accept_a_function_has_a_type_of_its
# _own_that_converts_to_its_shape.em`.)

fn add_one(x: int) -> int:
    return x + 1

fn sub_one(x: int) -> int:
    return x - 1

fn main():
    handler = add_one
    fn reset():
        handler = sub_one  #$ error[E2020]: each function has a type of its own
    reset()
    println(handler(1))
#$ help: declare the place `fn(i64) -> i64`, which holds any function of that shape
