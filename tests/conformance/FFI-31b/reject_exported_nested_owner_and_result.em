#$ test: compile-fail
#$ rules: FFI-31b
#$ profiles: debug
#$ error[E5015]: exported parameter `owner` reaches an owning Ember value
#$ error[E5015]: exported result reaches an owning Ember value

struct Owner:
    text: String

pub extern "C" fn return_text(owner: *Owner) -> String:
    return String()

fn main():
    pass
