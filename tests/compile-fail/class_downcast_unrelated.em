#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ error[E2020]: cannot downcast `A` to unrelated class `B`

class A:
    pass
class B:
    pass

fn main():
    value = A()
    bad: Option[B] = value as? B
