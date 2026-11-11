#$ test: run-pass
#$ rules: TYP-16, CLS-6, DRP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_6Marker4dropGda5cc12a188c7619b25d920e0657739a)

[T]
class Marker:
    value: i32
    payload: T

    fn drop():
        println(this.value)

fn main():
    _marker = Marker[bool](42, false)
