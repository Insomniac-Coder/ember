# The overlay of ../accept_out_parameters_beside_a_contracted_result.em.

unsafe overlay c "tokens.h":
    fn token_make(n, id: out one) -> returns_owned(destructor=token_free) one
    fn token_maybe(n, id: out one) -> returns_owned(destructor=token_free) one nullable
    fn token_live()
    hide token_free
