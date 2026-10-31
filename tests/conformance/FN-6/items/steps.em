## Functions another module uses as values
## (`accept_a_function_reached_through_its_module_is_a_value.em`).

pub fn offset(x: int) -> int:
    return x + 1

[F: needs fn(int) -> int]
pub fn twice(f: F, x: int) -> int:
    return f(f(x))

fn hidden(x: int) -> int:
    return x
