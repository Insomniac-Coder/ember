# The overlay of ../reject_an_entry_without_ffi_const_changes_its_object.em: an entry for a method
# that only reads its object, written without `@ffi(const)` (verdict 132).

unsafe overlay cpp "cache.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Cache:
        fn lookups()
