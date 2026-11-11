#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-16, TYP-22, CLO-6a

interface Consume[T]:
    fn consume() owns this:
        pass

fn take(x: ref dyn Consume[i32]):    #$ error[E2050]: `Consume[i32]` cannot be used as `dyn Consume[i32]`: `consume` takes its object with `owns`, and a `dyn` value cannot hand its object over
    pass
