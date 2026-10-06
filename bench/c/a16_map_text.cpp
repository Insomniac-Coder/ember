#include <cstdint>
#include <cstdio>
#include <string>
#include <unordered_map>
int main() {
    std::unordered_map<std::string, int64_t> m;
    for (int64_t i = 0; i < 200000; i++) m["key" + std::to_string(i)] = i;
    int64_t total = 0;
    for (int round = 0; round < 5; round++)
        for (int64_t i = 0; i < 200000; i++) total += m["key" + std::to_string(i)];
    std::printf("%zu %lld\n", m.size(), (long long)total);
    return 0;
}
