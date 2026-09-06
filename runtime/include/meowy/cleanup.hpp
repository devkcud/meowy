#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <span>
#include <string_view>

namespace meowy::prototype::v0 {

enum class Status : std::uint8_t { ok, full, invalid };
enum class Reason : std::uint8_t { complete, leave, restart, panic, cancel };

struct Panic final {
public:
    static constexpr std::size_t message_capacity = 256;
    Panic() noexcept = default;
    Panic(std::uint32_t code, std::string_view text) noexcept;

    std::uint32_t code = 0;
    [[nodiscard]] std::string_view message() const noexcept;
    [[nodiscard]] std::size_t original_size() const noexcept;
    [[nodiscard]] bool truncated() const noexcept;

private:
    std::array<char, message_capacity> bytes{};
    std::size_t length = 0;
    std::size_t original = 0;
};

using Drop = Panic (*)(void *) noexcept;

class Stack;

struct Token final {
public:
    const Stack *stack = nullptr;
    std::size_t index = 0;
    std::uint64_t id = 0;
};

struct Mark final {
public:
    const Stack *stack = nullptr;
    std::size_t depth = 0;
    std::uint64_t anchor = 0;
};

struct Slot final {
public:
    Status status = Status::invalid;
    Token token;
};

struct Outcome final {
public:
    Status status;
    Reason reason;
    Panic panic;
};

class Entry final {
public:
    Entry() = default;

private:
    friend class Stack;
    std::uint64_t id = 0;
    void *data = nullptr;
    Drop drop = nullptr;
    std::string_view name;
    bool initialized = false;
};

class Stack final {
public:
    explicit Stack(std::span<Entry> storage) noexcept;
    Stack(const Stack &) = delete;
    Stack &operator=(const Stack &) = delete;
    Stack(Stack &&) = delete;
    Stack &operator=(Stack &&) = delete;

    [[nodiscard]] Mark mark() const noexcept;
    [[nodiscard]] Slot reserve() noexcept;
    [[nodiscard]] Status arm(Token token, void *data, Drop drop,
                             std::string_view name) noexcept;
    [[nodiscard]] Status disarm(Token token) noexcept;
    [[nodiscard]] Slot transfer(Token token, Stack &destination) noexcept;
    [[nodiscard]] Outcome unwind(Mark mark, Reason reason,
                                 Panic panic = {}) noexcept;
    [[nodiscard]] std::size_t size() const noexcept;

private:
    [[nodiscard]] bool valid(Token token) const noexcept;
    [[nodiscard]] bool valid(Mark mark) const noexcept;

    std::span<Entry> entries;
    std::size_t depth = 0;
    std::uint64_t next = 1;
    bool busy = false;
};

}
