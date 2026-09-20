#$ test: run-pass
#$ rules: TYP-16, CLS-6, DRP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_Marker_bool_drop)

class Marker[T]:
    value: i32
    payload: T

    fn drop(mut self):
        println(self.value)

fn main():
    _marker = Marker[bool](42, false)
