#$ test: run-pass
#$ rules: FFI-15, TXT-2, TXT-5
#$ profiles: debug, release, shipping
#$ stdout: Ember
#$ stdout: invalid

fn main():
    match c"Ember".to_str():
        Ok(text) => println(text)
        Err(_) => println("unexpected")
    match c"\xFF".to_str():
        Ok(_) => println("unexpected")
        Err(_) => println("invalid")
