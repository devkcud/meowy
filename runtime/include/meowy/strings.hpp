#pragma once

#include "meowy/owned.hpp"

namespace meowy::prototype::v0 {

enum class StringStatus : std::uint32_t { ok, invalid, full, misaligned, occupied, allocation_failed };
enum class AllocationCause : std::uint32_t { none, exhausted };

struct AllocationFailure final {
public:
    AllocationCause cause = AllocationCause::none;
    std::uint64_t bytes = 0;
    std::uint64_t alignment = 0;
};

struct Allocator final {
public:
    void *const context;
    void *(*const allocate)(void *, std::size_t) noexcept;
    void (*const release)(void *, void *, std::size_t) noexcept;
};

class Strings final {
public:
    [[nodiscard]] static const Allocator &heap() noexcept;
    [[nodiscard]] static const ValueOps &ops() noexcept;
    [[nodiscard]] static StringStatus copy(Owned &owner, std::string_view text,
                                          const Allocator &allocator, AllocationFailure &failure) noexcept;
    [[nodiscard]] static bool view(const Owned &owner, std::string_view &text) noexcept;

private:
    struct Value final {
    public:
        const Allocator *allocator;
        char *data;
        std::size_t size;
    };

    static void move(void *destination, void *source) noexcept;
    static Panic drop(void *data) noexcept;
};

}

extern "C" {
const void *meowy_string_heap_v0() noexcept;
const void *meowy_string_ops_v0() noexcept;
std::uint64_t meowy_string_bytes_v0() noexcept;
std::uint64_t meowy_string_alignment_v0() noexcept;
std::uint32_t meowy_string_copy_v0(void *owner, const char *text, std::uint64_t size,
                                    const void *allocator,
                                    meowy::prototype::v0::AllocationFailure *failure) noexcept;
std::uint32_t meowy_string_view_v0(const void *owner, const char **text, std::uint64_t *size) noexcept;
}
