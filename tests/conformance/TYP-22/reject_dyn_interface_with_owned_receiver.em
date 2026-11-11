#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TYP-22, CLO-6a

interface Consume:
    fn consume() owns this:
        pass

fn take(x: ref dyn Consume):    #$ error[E2050]: `Consume` cannot be used as `dyn Consume`: `consume` takes its object with `owns`, and a `dyn` value cannot hand its object over
    pass
