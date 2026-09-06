#include <cerrno>
#include <cinttypes>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <unistd.h>

extern "C" void meowy_write_v1(int fd, const char *data, std::uint64_t size) {
    while (size != 0) {
        const std::uint64_t limit = size > 0x7ffff000 ? 0x7ffff000 : size;
        const ssize_t count = write(fd, data, static_cast<std::size_t>(limit));
        if (count < 0 && errno == EINTR) {
            continue;
        }
        if (count <= 0) {
            std::_Exit(1);
        }
        data += count;
        size -= static_cast<std::uint64_t>(count);
    }
}

extern "C" void meowy_int_v1(int fd, std::int64_t value) {
    char data[32];
    const int size = std::snprintf(data, sizeof(data), "%" PRId64, value);
    if (size < 0 || static_cast<std::size_t>(size) >= sizeof(data)) {
        std::_Exit(1);
    }
    meowy_write_v1(fd, data, static_cast<std::uint64_t>(size));
}

extern "C" void meowy_uint_v1(int fd, std::uint64_t value) {
    char data[32];
    const int size = std::snprintf(data, sizeof(data), "%" PRIu64, value);
    if (size < 0 || static_cast<std::size_t>(size) >= sizeof(data)) {
        std::_Exit(1);
    }
    meowy_write_v1(fd, data, static_cast<std::uint64_t>(size));
}

extern "C" void meowy_float_v1(int fd, double value, int bits) {
    char data[64];
    const int size = std::snprintf(data, sizeof(data), bits == 32 ? "%.9g" : "%.17g", value);
    if (size < 0 || static_cast<std::size_t>(size) >= sizeof(data)) {
        std::_Exit(1);
    }
    meowy_write_v1(fd, data, static_cast<std::uint64_t>(size));
}

extern "C" void meowy_bool_v1(int fd, bool value) {
    meowy_write_v1(fd, value ? "true" : "false", value ? 4 : 5);
}

extern "C" bool meowy_string_equal_v1(const char *left, std::uint64_t left_size,
                                      const char *right, std::uint64_t right_size) {
    return left_size == right_size && (left_size == 0 || std::memcmp(left, right, left_size) == 0);
}

extern "C" int meowy_string_compare_v1(const char *left, std::uint64_t left_size,
                                       const char *right, std::uint64_t right_size) {
    const std::uint64_t size = left_size < right_size ? left_size : right_size;
    const int order = size == 0 ? 0 : std::memcmp(left, right, size);
    if (order != 0) {
        return order;
    }
    return left_size < right_size ? -1 : left_size > right_size ? 1 : 0;
}

extern "C" [[noreturn]] void meowy_panic_v1() {
    meowy_write_v1(2, "\n", 1);
    std::_Exit(1);
}

extern "C" [[noreturn]] void meowy_arithmetic_fail_v1() {
    constexpr char message[] = "panic[P002]: integer overflow or division by zero";
    meowy_write_v1(2, message, sizeof(message) - 1);
    meowy_panic_v1();
}
