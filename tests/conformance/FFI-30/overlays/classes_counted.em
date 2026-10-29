# A third overlay of classes.hpp (D-879), for a reject case: an entry for a member of a class
# another import of the header wrote.

unsafe overlay cpp "classes.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn doubled(n)
    class Counter:
        fn count()
