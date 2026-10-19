#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-22, CLO-6a

interface Consume:
    fn consume(owned self):
        pass

fn take(x: ref dyn Consume):    #$ error[E2050]: method `consume` has an owned receiver
    pass
