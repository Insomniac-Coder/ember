#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6, OWN-5
#$ profiles: debug, release, shipping
#$ assert-c: contains(ember_obj_new(&em_ti_Defaults))
#$ assert-c: contains(em_Defaults_init)
#$ stdout: true
#$ stdout: 5
#$ stdout: true

class Defaults:
    retries: i32 = 3
    enabled: bool = true

    fn init(mut self, retries: i32):
        println(self.enabled)
        self.retries = retries

fn main():
    value = Defaults(5)
    println(value.retries)
    println(value.enabled)
