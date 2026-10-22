# [CXX-1] — the corpus program: nlohmann/json through its own header and the two functions of
# `json_api.hpp`. A parse gives a document a `ForeignBox` owns (an opaque class by value, ODR-166);
# the instantiation's methods read it; its `dump` gives a `std::string`; each C++ exception comes
# back as a `CppError`. Only `what()` is printed: without RTTI a thrown class has no name.

from std.ffi import CppError
import cpp "json_api.hpp" with (project = "json", instantiate = ["nlohmann::basic_json<>"])

fn main():
    text = "{\"name\": \"ember\", \"tags\": [1, 2, 3]}".as_bytes()
    tags = "tags".as_bytes()
    missing = "missing".as_bytes()
    broken = "{bad".as_bytes()
    unsafe:  # SAFETY(ffi): each pointer and length is a live string's bytes, read during the call
        match cpp.json_api.parse(text.as_ptr(), text.len() as u64):
            Ok(document):
                json = document.get()
                println(json.is_object(), json.size())
                match cpp.json_api.count(json, tags.as_ptr(), tags.len() as u64):
                    Ok(n):
                        println(n)
                    Err(error):
                        println(error.what)
                match cpp.json_api.count(json, missing.as_ptr(), missing.len() as u64):
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
        match cpp.json_api.parse(broken.as_ptr(), broken.len() as u64):
            Ok(_):
                println("parsed")
            Err(error):
                println(error.what)
