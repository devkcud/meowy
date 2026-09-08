#pragma once

#include "meowy/cleanup.hpp"

#include <span>

namespace meowy::prototype::v0 {

enum class OwnedStatus : unsigned char { ok, invalid, full, misaligned, occupied, overlap };

struct ValueOps final {
public:
    const std::size_t size;
    const std::size_t alignment;
    void (*const move)(void *, void *) noexcept;
    const Drop drop;
    const std::string_view name;
    void (*const capture)(void *, void *) noexcept = nullptr;
};

class Scheduler;
class Generated;
class Strings;

class Owned final {
public:
    explicit Owned(std::span<std::byte> storage = {}) noexcept;
    Owned(const Owned &) = delete;
    Owned &operator=(const Owned &) = delete;
    Owned(Owned &&) = delete;
    Owned &operator=(Owned &&) = delete;

    [[nodiscard]] OwnedStatus reserve(const ValueOps &ops) noexcept;
    [[nodiscard]] OwnedStatus commit() noexcept;
    [[nodiscard]] OwnedStatus move_to(Owned &destination) noexcept;
    [[nodiscard]] OwnedStatus release() noexcept;
    [[nodiscard]] bool initialized() const noexcept;
    [[nodiscard]] bool empty() const noexcept;
    [[nodiscard]] void *data() noexcept;
    [[nodiscard]] const void *data() const noexcept;

private:
    friend class Scheduler;
    friend class Generated;
    friend class Strings;
    enum class Phase : unsigned char { empty, reserved, live };
    [[nodiscard]] OwnedStatus accepts(const ValueOps &ops) const noexcept;
    [[nodiscard]] OwnedStatus fits(const Owned &destination) const noexcept;
    static Panic drop(void *data) noexcept;

    const std::span<std::byte> bytes;
    const ValueOps *ops = nullptr;
    Phase phase = Phase::empty;
    bool busy = false;
};

}
