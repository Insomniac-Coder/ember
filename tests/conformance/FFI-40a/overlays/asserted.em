# The overlay of ../accept_ffi_const_asserts_a_method_only_reads_its_object.em.

unsafe overlay cpp "ledger.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Ledger:
        @ffi(const)
        fn balance()
