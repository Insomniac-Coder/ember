#$ rules: DIA-12, MOD-2, MOD-3
#$ test: compile-fail
#$ error[E1010]: cannot find `support.package_api.pritn` in this scope
#$ help: the nearest name is `api.print`
import support.package_api as api

fn main():
    api.pritn(1)
