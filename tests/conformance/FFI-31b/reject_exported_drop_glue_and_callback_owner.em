#$ test: compile-fail
#$ rules: FFI-31b
#$ profiles: debug
#$ error[E5015]: exported parameter `ticket` reaches an owning Ember value
#$ error[E5015]: exported parameter `callback` reaches an owning Ember value

struct Ticket:
    id: i32

    fn drop(mut self):
        pass

pub extern "C" fn inspect_ticket(ticket: *Ticket) -> i32:
    return 0

pub extern "C" fn call_back(callback: extern "C" fn(*String) -> i32) -> i32:
    return 0

fn main():
    pass
