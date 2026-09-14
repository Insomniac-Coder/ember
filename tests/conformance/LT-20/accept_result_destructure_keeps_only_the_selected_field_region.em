#$ test: run-pass
#$ rules: LT-17, LT-20, LT-24, LT-36, LT-38, TYP-15, TST-17
#$ stdout: 17

# `Result` uses the same ordinary enum-payload flow as `Option`: an explicit
# nested `Ok` destructure carries only the selected field's provenance.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(17)
    right: Array[i32] = Array[i32]()
    right.push(18)
    result: Result[Pair, i32] = Ok(Pair(left.as_span(), right.as_span()))
    right.push(19)
    match result:
        Ok(Pair(selected, _)):
            println(selected[0])
        Err(_):
            pass
