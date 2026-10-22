## An interface with a default method, in a module the program loads before the module implementing
## it (`accept_a_default_method_reaches_an_implementation_in_a_later_module.em`).

pub interface Named:
    fn name(self) -> str

    fn greeting(self) -> String:
        return f"hello, {self.name()}"
