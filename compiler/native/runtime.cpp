#include "meowy/cleanup.hpp"

#include <cerrno>
#include <cinttypes>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <new>
#include <unistd.h>

using meowy::prototype::v0::Panic;

static void output(int fd, const char *data, std::uint64_t size, Panic *panic) {
    if (panic != nullptr) {
        panic->append({data, static_cast<std::size_t>(size)});
    }
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

extern "C" void meowy_write_v1(int fd, const char *data, std::uint64_t size) {
    output(fd, data, size, nullptr);
}

extern "C" void meowy_panic_text_v0(Panic *panic, const char *data, std::uint64_t size) {
    output(2, data, size, panic);
}

extern "C" void meowy_panic_begin_v0(Panic *panic, std::uint32_t code) {
    new (panic) Panic{code, {}};
}

extern "C" void meowy_panic_copy_v0(Panic *destination, const Panic *source) {
    *destination = *source;
}

static void int_value(int fd, std::int64_t value, Panic *panic) {
    char data[32];
    const int size = std::snprintf(data, sizeof(data), "%" PRId64, value);
    if (size < 0 || static_cast<std::size_t>(size) >= sizeof(data)) {
        std::_Exit(1);
    }
    output(fd, data, static_cast<std::uint64_t>(size), panic);
}

extern "C" void meowy_int_v1(int fd, std::int64_t value) {
    int_value(fd, value, nullptr);
}

extern "C" void meowy_panic_int_v0(Panic *panic, std::int64_t value) {
    int_value(2, value, panic);
}

static void uint_value(int fd, std::uint64_t value, Panic *panic) {
    char data[32];
    const int size = std::snprintf(data, sizeof(data), "%" PRIu64, value);
    if (size < 0 || static_cast<std::size_t>(size) >= sizeof(data)) {
        std::_Exit(1);
    }
    output(fd, data, static_cast<std::uint64_t>(size), panic);
}

extern "C" void meowy_uint_v1(int fd, std::uint64_t value) {
    uint_value(fd, value, nullptr);
}

extern "C" void meowy_panic_uint_v0(Panic *panic, std::uint64_t value) {
    uint_value(2, value, panic);
}

static void float_value(int fd, double value, int bits, Panic *panic) {
    char data[64];
    const int size = std::snprintf(data, sizeof(data), bits == 32 ? "%.9g" : "%.17g", value);
    if (size < 0 || static_cast<std::size_t>(size) >= sizeof(data)) {
        std::_Exit(1);
    }
    output(fd, data, static_cast<std::uint64_t>(size), panic);
}

extern "C" void meowy_float_v1(int fd, double value, int bits) {
    float_value(fd, value, bits, nullptr);
}

extern "C" void meowy_panic_float_v0(Panic *panic, double value, int bits) {
    float_value(2, value, bits, panic);
}

extern "C" void meowy_bool_v1(int fd, bool value) {
    meowy_write_v1(fd, value ? "true" : "false", value ? 4 : 5);
}

extern "C" void meowy_panic_bool_v0(Panic *panic, bool value) {
    meowy_panic_text_v0(panic, value ? "true" : "false", value ? 4 : 5);
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

static void site(const std::uint64_t start, const std::uint64_t end, Panic *panic) {
    constexpr char text[] = " at bytes ";
    meowy_panic_text_v0(panic, text, sizeof(text) - 1);
    meowy_panic_uint_v0(panic, start);
    meowy_panic_text_v0(panic, "..", 2);
    meowy_panic_uint_v0(panic, end);
}

extern "C" [[noreturn]] void meowy_panic_site_v1(const std::uint64_t start,
                                                const std::uint64_t end) {
    site(start, end, nullptr);
    meowy_panic_v1();
}

extern "C" void meowy_panic_site_v0(Panic *panic, const std::uint64_t start,
                                      const std::uint64_t end) {
    site(start, end, panic);
    meowy_write_v1(2, "\n", 1);
}

static void operand(const std::uint64_t value, const int is_signed, Panic *panic) {
    if (is_signed != 0) {
        meowy_panic_int_v0(panic, static_cast<std::int64_t>(value));
    } else {
        meowy_panic_uint_v0(panic, value);
    }
}

extern "C" void meowy_arithmetic_capture_v0(Panic *panic, const int op,
                                                     const int bits,
                                                     const int is_signed,
                                                     const std::uint64_t left,
                                                     const std::uint64_t right,
                                                     const std::uint64_t start,
                                                     const std::uint64_t end) {
    meowy_panic_begin_v0(panic, 2);
    if (bits < 1 || bits > 64) {
        meowy_arithmetic_fail_v1();
    }
    constexpr char prefix[] = "panic[P002]: ";
    meowy_write_v1(2, prefix, sizeof(prefix) - 1);
    meowy_panic_text_v0(panic, is_signed != 0 ? "int" : "uint", is_signed != 0 ? 3 : 4);
    meowy_panic_uint_v0(panic, static_cast<std::uint64_t>(bits));
    meowy_panic_text_v0(panic, " ", 1);
    if (op == 0) {
        meowy_panic_text_v0(panic, "unary -", 7);
    } else {
        const char symbol = static_cast<char>(op);
        meowy_panic_text_v0(panic, &symbol, 1);
    }
    const bool zero = (op == '/' || op == '%') && right == 0;
    meowy_panic_text_v0(panic, zero ? " zero divisor" : " overflow", zero ? 13 : 9);
    meowy_panic_text_v0(panic, op == 0 ? " (value " : " (left ", op == 0 ? 8 : 7);
    operand(left, is_signed, panic);
    if (op != 0) {
        meowy_panic_text_v0(panic, ", right ", 8);
        operand(right, is_signed, panic);
    }
    meowy_panic_text_v0(panic, "; range ", 8);
    if (is_signed != 0) {
        const std::uint64_t limit = std::uint64_t{1} << (bits - 1);
        meowy_panic_int_v0(panic, -static_cast<std::int64_t>(limit - 1) - 1);
        meowy_panic_text_v0(panic, "..", 2);
        meowy_panic_uint_v0(panic, limit - 1);
    } else {
        meowy_panic_text_v0(panic, "0..", 3);
        meowy_panic_uint_v0(panic, UINT64_MAX >> (64 - bits));
    }
    meowy_panic_text_v0(panic, ")", 1);
    meowy_panic_site_v0(panic, start, end);
}

extern "C" void meowy_index_capture_v0(Panic *panic, const std::uint64_t index,
                                                const std::uint64_t length,
                                                const int signed_index,
                                                const std::uint64_t start,
                                                const std::uint64_t end) {
    meowy_panic_begin_v0(panic, 1);
    constexpr char prefix[] = "panic[P001]: ";
    constexpr char suffix[] = " is outside initialized length ";
    meowy_write_v1(2, prefix, sizeof(prefix) - 1);
    meowy_panic_text_v0(panic, "index ", 6);
    if (signed_index != 0) {
        meowy_panic_int_v0(panic, static_cast<std::int64_t>(index));
    } else {
        meowy_panic_uint_v0(panic, index);
    }
    meowy_panic_text_v0(panic, suffix, sizeof(suffix) - 1);
    meowy_panic_uint_v0(panic, length);
    meowy_panic_site_v0(panic, start, end);
}

extern "C" void meowy_list_capture_v0(Panic *panic, const std::uint64_t length,
                                               const std::uint64_t capacity,
                                               const std::uint64_t start,
                                               const std::uint64_t end) {
    meowy_panic_begin_v0(panic, 3);
    constexpr char prefix[] = "panic[P003]: ";
    constexpr char middle[] = ", capacity ";
    meowy_write_v1(2, prefix, sizeof(prefix) - 1);
    constexpr char message[] = "bounded list is full (length ";
    meowy_panic_text_v0(panic, message, sizeof(message) - 1);
    meowy_panic_uint_v0(panic, length);
    meowy_panic_text_v0(panic, middle, sizeof(middle) - 1);
    meowy_panic_uint_v0(panic, capacity);
    meowy_panic_text_v0(panic, ")", 1);
    meowy_panic_site_v0(panic, start, end);
}

extern "C" [[noreturn]] void meowy_arithmetic_fail_v2(const int op, const int bits,
    const int is_signed, const std::uint64_t left, const std::uint64_t right,
    const std::uint64_t start, const std::uint64_t end) {
    Panic panic;
    meowy_arithmetic_capture_v0(&panic, op, bits, is_signed, left, right, start, end);
    std::_Exit(1);
}

extern "C" [[noreturn]] void meowy_index_fail_v1(const std::uint64_t index,
    const std::uint64_t length, const int signed_index, const std::uint64_t start,
    const std::uint64_t end) {
    Panic panic;
    meowy_index_capture_v0(&panic, index, length, signed_index, start, end);
    std::_Exit(1);
}

extern "C" [[noreturn]] void meowy_list_full_v1(const std::uint64_t length,
    const std::uint64_t capacity, const std::uint64_t start, const std::uint64_t end) {
    Panic panic;
    meowy_list_capture_v0(&panic, length, capacity, start, end);
    std::_Exit(1);
}
