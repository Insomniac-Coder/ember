#include <cstdint>
#include <cstdio>
#include <unordered_map>
static int64_t key(int64_t i) { return (int64_t)((uint64_t)i * 0x5851F42D4C957F2DULL) >> 8; }
int main() {
    std::unordered_map<int64_t, int64_t> m;
    for (int64_t i = 0; i < 1000000; i++) m.insert({key(i), i});
    int64_t total = 0;
    for (int round = 0; round < 5; round++)
        for (int64_t i = 0; i < 1000000; i++) total += m.at(key(i));
    std::printf("%lld %lld\n", (long long)m.size(), (long long)total);
    return 0;
}
