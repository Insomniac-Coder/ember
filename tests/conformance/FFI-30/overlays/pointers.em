# The overlay of ../accept_two_declarations_of_one_function_keep_their_contracts.em's first import.

unsafe overlay c "pointers.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn pointers_read(p: borrowed one)
    fn pointers_bump(p: borrowed one)
