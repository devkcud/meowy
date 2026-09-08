#include "meowy/cleanup.hpp"

#include <cerrno>
#include <charconv>
#include <cstdlib>
#include <cstring>
#include <limits>
#include <unistd.h>

namespace meowy::prototype::v0 {
namespace {

void write(std::string_view text) noexcept {
    while (!text.empty()) {
        const auto size = text.size() > 4096 ? 4096 : text.size();
        const auto count = ::write(STDERR_FILENO, text.data(), size);
        if (count < 0 && errno == EINTR) {
            continue;
        }
        if (count <= 0) {
            return;
        }
        text.remove_prefix(static_cast<std::size_t>(count));
    }
}

void write(const Panic &panic) noexcept {
    char code[10];
    const auto result = std::to_chars(code, code + sizeof(code), panic.code);
    write("P");
    if (result.ptr - code < 3) {
        write(result.ptr - code == 1 ? "00" : "0");
    }
    write({code, static_cast<std::size_t>(result.ptr - code)});
    write(": ");
    write(panic.message());
    if (panic.truncated()) {
        char size[20];
        const auto count = std::to_chars(size, size + sizeof(size), panic.original_size());
        write(" [truncated from ");
        write({size, static_cast<std::size_t>(count.ptr - size)});
        write(" bytes]");
    }
}

std::string_view name(Reason reason) noexcept {
    switch (reason) {
    case Reason::complete:
        return "complete";
    case Reason::leave:
        return "leave";
    case Reason::restart:
        return "restart";
    case Reason::panic:
        return "panic";
    case Reason::cancel:
        return "cancel";
    }
    return "invalid";
}

[[noreturn]] void fatal(Reason reason, const Panic &first, std::string_view operation,
                       const Panic &second) noexcept {
    write("panic[P008]: panic during cleanup\noriginal: ");
    write(name(reason));
    if (first.code != 0) {
        write(" ");
        write(first);
    }
    write("\ncleanup: ");
    write(operation);
    write("\nsecond: ");
    write(second);
    write("\n");
    std::abort();
}

}

Panic::Panic(std::uint32_t value, std::string_view text) noexcept
    : code(value), length(text.size() < message_capacity ? text.size() : message_capacity), original(text.size()) {
    if (length < original) {
        while (length != 0 && (static_cast<unsigned char>(text[length]) & 0xc0) == 0x80) {
            --length;
        }
    }
    if (length != 0) {
        std::memcpy(bytes.data(), text.data(), length);
    }
}

std::string_view Panic::message() const noexcept {
    return {bytes.data(), length};
}

std::size_t Panic::original_size() const noexcept {
    return original;
}

bool Panic::truncated() const noexcept {
    return length < original;
}

Stack::Stack(std::span<Entry> storage) noexcept : entries(storage) {}

Mark Stack::mark() const noexcept {
    return {this, depth, depth == 0 ? 0 : entries[depth - 1].id};
}

Slot Stack::reserve() noexcept {
    if (busy) {
        return {};
    }
    if (depth == entries.size() || next == std::numeric_limits<std::uint64_t>::max()) {
        return {Status::full, {}};
    }
    const auto index = depth++;
    entries[index] = {};
    entries[index].id = next++;
    return {Status::ok, {this, index, entries[index].id}};
}

Status Stack::arm(Token token, void *data, Drop drop, std::string_view name) noexcept {
    if (!valid(token) || token.index + 1 != depth || entries[token.index].initialized ||
        drop == nullptr || name.empty()) {
        return Status::invalid;
    }
    auto &entry = entries[token.index];
    entry.data = data;
    entry.drop = drop;
    entry.name = name;
    entry.initialized = true;
    return Status::ok;
}

Status Stack::disarm(Token token) noexcept {
    if (!valid(token) || entries[token.index].drop == nullptr) {
        return Status::invalid;
    }
    entries[token.index].drop = nullptr;
    return Status::ok;
}

Slot Stack::transfer(Token token, Stack &destination) noexcept {
    if (this == &destination || !valid(token) || entries[token.index].drop == nullptr) {
        return {};
    }
    const auto slot = destination.reserve();
    if (slot.status != Status::ok) {
        return slot;
    }
    const auto &source = entries[token.index];
    auto &entry = destination.entries[slot.token.index];
    entry.data = source.data;
    entry.drop = source.drop;
    entry.name = source.name;
    entry.initialized = true;
    entries[token.index].drop = nullptr;
    return slot;
}

Status Stack::can_rebind(Token token, const Stack &destination, Token reserved) const noexcept {
    if (!valid(token) || entries[token.index].drop == nullptr || !destination.valid(reserved) ||
        reserved.index + 1 != destination.depth || destination.entries[reserved.index].initialized) {
        return Status::invalid;
    }
    return Status::ok;
}

Outcome Stack::unwind(Mark mark, Reason reason, Panic panic) noexcept {
    if (!valid(mark) || name(reason) == "invalid" || (reason == Reason::panic) != (panic.code != 0)) {
        return {Status::invalid, reason, panic};
    }
    busy = true;
    while (depth > mark.depth) {
        const auto entry = entries[--depth];
        entries[depth] = {};
        if (entry.drop != nullptr) {
            const auto second = entry.drop(entry.data);
            if (second.code != 0) {
                fatal(reason, panic, entry.name, second);
            }
        }
    }
    busy = false;
    return {Status::ok, reason, panic};
}

std::size_t Stack::size() const noexcept {
    return depth;
}

bool Stack::valid(Token token) const noexcept {
    return !busy && token.stack == this && token.index < depth &&
           token.id != 0 && token.id == entries[token.index].id;
}

bool Stack::valid(Mark mark) const noexcept {
    return !busy && mark.stack == this && mark.depth <= depth &&
           (mark.depth == 0 ? mark.anchor == 0 : mark.anchor == entries[mark.depth - 1].id);
}

}
