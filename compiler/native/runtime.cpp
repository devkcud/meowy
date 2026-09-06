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

static void site(const std::uint64_t start, const std::uint64_t end) {
    constexpr char text[] = " at bytes ";
    meowy_write_v1(2, text, sizeof(text) - 1);
    meowy_uint_v1(2, start);
    meowy_write_v1(2, "..", 2);
    meowy_uint_v1(2, end);
}

extern "C" [[noreturn]] void meowy_panic_site_v1(const std::uint64_t start,
                                                const std::uint64_t end) {
    site(start, end);
    meowy_panic_v1();
}

static void operand(const std::uint64_t value, const int is_signed) {
    if (is_signed != 0) {
        meowy_int_v1(2, static_cast<std::int64_t>(value));
    } else {
        meowy_uint_v1(2, value);
    }
}

extern "C" [[noreturn]] void meowy_arithmetic_fail_v2(const int op,
                                                     const int bits,
                                                     const int is_signed,
                                                     const std::uint64_t left,
                                                     const std::uint64_t right,
                                                     const std::uint64_t start,
                                                     const std::uint64_t end) {
    if (bits < 1 || bits > 64) {
        meowy_arithmetic_fail_v1();
    }
    constexpr char prefix[] = "panic[P002]: ";
    meowy_write_v1(2, prefix, sizeof(prefix) - 1);
    meowy_write_v1(2, is_signed != 0 ? "int" : "uint", is_signed != 0 ? 3 : 4);
    meowy_uint_v1(2, static_cast<std::uint64_t>(bits));
    meowy_write_v1(2, " ", 1);
    if (op == 0) {
        meowy_write_v1(2, "unary -", 7);
    } else {
        const char symbol = static_cast<char>(op);
        meowy_write_v1(2, &symbol, 1);
    }
    const bool zero = (op == '/' || op == '%') && right == 0;
    meowy_write_v1(2, zero ? " zero divisor" : " overflow", zero ? 13 : 9);
    meowy_write_v1(2, op == 0 ? " (value " : " (left ", op == 0 ? 8 : 7);
    operand(left, is_signed);
    if (op != 0) {
        meowy_write_v1(2, ", right ", 8);
        operand(right, is_signed);
    }
    meowy_write_v1(2, "; range ", 8);
    if (is_signed != 0) {
        const std::uint64_t limit = std::uint64_t{1} << (bits - 1);
        meowy_int_v1(2, -static_cast<std::int64_t>(limit - 1) - 1);
        meowy_write_v1(2, "..", 2);
        meowy_uint_v1(2, limit - 1);
    } else {
        meowy_write_v1(2, "0..", 3);
        meowy_uint_v1(2, UINT64_MAX >> (64 - bits));
    }
    meowy_write_v1(2, ")", 1);
    meowy_panic_site_v1(start, end);
}

extern "C" [[noreturn]] void meowy_index_fail_v1(const std::uint64_t index,
                                                const std::uint64_t length,
                                                const int signed_index,
                                                const std::uint64_t start,
                                                const std::uint64_t end) {
    constexpr char prefix[] = "panic[P001]: index ";
    constexpr char suffix[] = " is outside initialized length ";
    meowy_write_v1(2, prefix, sizeof(prefix) - 1);
    if (signed_index != 0) {
        meowy_int_v1(2, static_cast<std::int64_t>(index));
    } else {
        meowy_uint_v1(2, index);
    }
    meowy_write_v1(2, suffix, sizeof(suffix) - 1);
    meowy_uint_v1(2, length);
    site(start, end);
    meowy_panic_v1();
}

extern "C" [[noreturn]] void meowy_list_full_v1(const std::uint64_t length,
                                               const std::uint64_t capacity,
                                               const std::uint64_t start,
                                               const std::uint64_t end) {
    constexpr char prefix[] = "panic[P003]: bounded list is full (length ";
    constexpr char middle[] = ", capacity ";
    meowy_write_v1(2, prefix, sizeof(prefix) - 1);
    meowy_uint_v1(2, length);
    meowy_write_v1(2, middle, sizeof(middle) - 1);
    meowy_uint_v1(2, capacity);
    meowy_write_v1(2, ")", 1);
    site(start, end);
    meowy_panic_v1();
}
