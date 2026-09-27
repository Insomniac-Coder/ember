#$ test: compile-fail
#$ rules: FFI-31b
#$ profiles: debug
#$ error[E5015]: exported parameter `text` reaches an owning Ember value

pub extern "C" fn read_text(text: *String) -> i32:
    return 0

fn main():
    pass
