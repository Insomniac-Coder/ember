#$ test: parse-pass
#$ rules: FFI-10, FFI-8, STA-1
# `[FFI-10]` — a foreign static is a place in C's storage, whatever C value
# it holds: a record that is not `Copy` is read and written field by field and
# assigned whole, an element of an array in a static is written in place, and
# a whole array and a C function pointer are globals too. The C side is
# `ember_driver/tests/fixtures/foreign_statics.c`; the milestone
# `foreign_statics_are_places_in_c_storage` links and runs this program.

struct Account:
    id: i32
    balance: i64

@derive(Copy)
struct Table:
    values: [i32; 3]
    total: i32

unsafe extern "C":
    static mut account: Account
    static mut table: Table
    static mut samples: [i32; 4]
    @ffi(immutable)
    static frozen_samples: [i32; 4]
    static mut scale: extern "C" fn(i32) -> i32
    safe fn account_sum() -> i64
    safe fn table_sum() -> i32
    safe fn samples_sum() -> i32
    fn bump_table_total() -> i32

fn main():
    unsafe:  # SAFETY(ffi): each global is declared as the fixture defines it
        println(account.id)
        account.balance = 250
        println(account_sum())
        account = Account(id = 9, balance = 1000)
        println(account_sum())
        account.balance += 1
        println(account.balance)
        table.values[1] = 40
        println(table_sum())
        table.values[2] += bump_table_total()
        println(table_sum())
        samples[3] = 20
        samples[0] *= 3
        println(samples_sum())
        println(samples[0] + samples[3])
        triple = scale
        println(triple(7))
    println(frozen_samples[1] + frozen_samples[2])
