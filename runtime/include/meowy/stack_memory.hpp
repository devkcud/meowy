#pragma once

#include <cstddef>

namespace meowy::prototype::v0 {

enum class MemoryStatus : unsigned char {
    ok,
    invalid,
    overflow,
    system_failed,
    allocation_failed,
    protection_failed,
    release_failed,
};

struct MemoryResult final {
public:
    MemoryStatus status = MemoryStatus::ok;
    int error = 0;
    int rollback_error = 0;
};

struct StackSize final {
public:
    std::size_t usable = 0;
    std::size_t guard = 0;
    std::size_t total = 0;
};

struct StackPlan final {
public:
    MemoryResult result;
    StackSize size;
};

class StackMemory final {
public:
    StackMemory() = default;
    StackMemory(const StackMemory &) = delete;
    StackMemory &operator=(const StackMemory &) = delete;
    StackMemory(StackMemory &&) = delete;
    StackMemory &operator=(StackMemory &&) = delete;

    [[nodiscard]] static StackPlan plan(std::size_t bytes, std::size_t page) noexcept;
    [[nodiscard]] MemoryResult allocate(std::size_t bytes) noexcept;
    [[nodiscard]] MemoryResult release() noexcept;
    [[nodiscard]] bool owns() const noexcept;
    [[nodiscard]] std::byte *data() noexcept;
    [[nodiscard]] const std::byte *data() const noexcept;
    [[nodiscard]] StackSize size() const noexcept;

private:
    void *base = nullptr;
    StackSize layout;
    bool owned = false;
    bool ready = false;
};

}
