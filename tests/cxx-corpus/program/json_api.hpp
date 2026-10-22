#pragma once
// [CXX-1] — the corpus program's own C++: two functions that reach nlohmann/json by types Ember
// has, as `W5002`'s help says to write for what a header-only library offers only as templates and
// overloads (`json::parse`, `json::at`).
#include <cstddef>
#include <string>
#include "nlohmann/json.hpp"

namespace json_api {

// The document the `length` bytes at `text` hold; throws `nlohmann::json::parse_error` for none.
inline nlohmann::json parse(const unsigned char* text, std::size_t length) {
    return nlohmann::json::parse(text, text + length);
}

// How many elements the member named by the `length` bytes at `key` has; throws
// `nlohmann::json::out_of_range` when the object has no such member.
inline std::size_t count(const nlohmann::json& document, const unsigned char* key, std::size_t length) {
    return document.at(std::string(reinterpret_cast<const char*>(key), length)).size();
}

}  // namespace json_api
