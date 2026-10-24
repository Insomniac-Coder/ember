fn needs_send<T: Send>(_t: T) {}
fn main() { needs_send(leak_lib::make()); }
