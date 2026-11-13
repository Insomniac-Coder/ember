#! threads creator
#$ test: parse-pass
#$ rules: GRM-37, FFI-33
#$ profiles: debug, development, release

# SAFETY: nothing else in the process defines this name
@unsafe(export(threads=creator))
fn creator_export() -> i32:
    return 0

fn main():
    pass
