# A module of accept_a_declaration_of_a_function_the_program_defines.em and
# reject_declarations_of_one_function_that_disagree.em: it defines the C function `ffi30_add`.

pub extern "C" fn ffi30_add(a: i32, b: i32) -> i32:
    return a + b
