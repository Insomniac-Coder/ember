#$ test: parse-pass
#$ rules: GRM-2
# A library module with a statement at file scope; only an entry file is a script.

pub fn answer() -> int:
    return 42

println("not here")
