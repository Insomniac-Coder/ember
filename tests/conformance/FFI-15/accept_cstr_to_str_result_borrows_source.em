#$ test: run-pass
#$ rules: FFI-15, TXT-2, LT-1, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: borrowed

fn checked(text: cstr) -> str:
    match text.to_str():
        Ok(value):
            return value
        Err(_):
            panic("invalid UTF-8")

fn main():
    println(checked(c"borrowed"))
