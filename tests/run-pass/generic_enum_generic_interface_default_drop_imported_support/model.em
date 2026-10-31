from generic_enum_generic_interface_default_drop_imported_support.traits import Answer

pub struct Token:
    pub value: i32

    fn drop():
        println(this.value)

pub enum Signal[T] implements Answer[T]:
    Ready(value: T)
