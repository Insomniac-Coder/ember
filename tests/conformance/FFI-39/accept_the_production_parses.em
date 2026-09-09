#$ test: parse-pass
#$ rules: FFI-39
# `[FFI-39]`'s declared foreign base had no production until errata ERR-037
# added one. It parses; nothing past the parser understands it yet, which is
# what the reject case beside this one records.

@ffi(trampoline, virtuals=["OnAttach", "OnUpdate"])
extern class cpp.RageV.Layer:
    pass

fn main():
    println(1)
