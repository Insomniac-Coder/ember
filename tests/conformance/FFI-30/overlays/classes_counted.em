# A third overlay of classes.hpp (D-879), for a reject case: an entry for a member of a class
# another import of the header wrote.

unsafe overlay cpp "classes.hpp":
    fn count()
    fn doubled(n)
