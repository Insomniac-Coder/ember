#include <cstdint>
#include <cstdio>
#include <unordered_map>
int main() {
    std::unordered_map<int64_t, int64_t> m;
    for (int64_t i = 0; i < 1000000; i++) m.insert({i * 7, i});
    int64_t total = 0;
    for (int round = 0; round < 5; round++)
        for (int64_t i = 0; i < 1000000; i++) total += m.at(i * 7);
    std::printf("%lld %lld\n", (long long)m.size(), (long long)total);
    return 0;
}
