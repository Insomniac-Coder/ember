# [CXX-1] — the corpus program: nlohmann/json through its own header and the two functions of
# `json_api.hpp`. Its overlay (`json_api.overlay.em`) states what the headers cannot, so no call is
# `unsafe`: each text is a `Span[u8]`, which the call passes as its pointer and length. A parse
# gives a document a `ForeignBox` owns (an opaque class by value, ODR-166); the instantiation's
# methods read it; its `dump` gives a `std::string`; each C++ exception comes back as a `CppError`.
# Only `what()` is printed: without RTTI a thrown class has no name.

from std.ffi import CppError
import cpp "json_api.hpp" with (project = "json", instantiate = ["nlohmann::basic_json<>"], overlay = "json_api.overlay.em")

fn main():
    match cpp.json_api.parse("{\"name\": \"ember\", \"tags\": [1, 2, 3]}".as_bytes()):
        Ok(document):
            json = document.get()
            println(json.is_object(), json.size())
            match cpp.json_api.count(json, "tags".as_bytes()):
                Ok(n):
                    println(n)
                Err(error):
                    println(error.what)
            match cpp.json_api.count(json, "missing".as_bytes()):
                Ok(n):
                    println(n)
                Err(error):
                    println(error.what)
            match json.dump(-1, 32, false, cpp.nlohmann.detail.error_handler_t.strict):
                Ok(dumped):
                    println(String.from(dumped))
                Err(error):
                    println(error.what)
        Err(error):
            println(error.what)
    match cpp.json_api.parse("{bad".as_bytes()):
        Ok(_):
            println("parsed")
        Err(error):
            println(error.what)
