# The overlay of ../reject_a_member_entry_that_differs_from_the_header.em: member entries that do not
# match the header's members.

unsafe overlay cpp "cache.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Cache:
        fn lookups(key)
        fn add(amount)
        fn missing()
    class Nothing:
        fn lookups()
