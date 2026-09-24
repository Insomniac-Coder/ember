#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TYP-16, TYP-22, CLO-6a

interface Consume[T]:
    fn consume(owned self):
        pass

fn take(x: ref dyn Consume[i32]):    #$ error[E2050]: method `consume` has an owned receiver
    pass
