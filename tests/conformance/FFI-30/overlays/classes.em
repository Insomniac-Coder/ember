# An overlay of classes.hpp (D-879): the class's members and the function, asserted safe.

unsafe overlay cpp "classes.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn doubled(n)
    class Counter:
        fn new()
        fn add(n)
        fn count()
