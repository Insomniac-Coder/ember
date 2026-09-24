## `[MOD-2]` — a module with one private and one public function.
fn helper() -> int:
    return 1

pub fn api() -> int:
    return helper() + 1
