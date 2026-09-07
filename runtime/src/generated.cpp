#include "meowy/generated.hpp"
#include "meowy/cleanup.hpp"

#include <limits>
#include <memory>
#include <type_traits>

namespace {
using namespace meowy::prototype::v0;

struct Record final {
public:
    void *data = nullptr;
    MeowyCleanupDrop drop = nullptr;
};

class Frame final {
public:
    explicit Frame(std::uint64_t count) noexcept
        : capacity(count), records(reinterpret_cast<Record *>(reinterpret_cast<std::byte *>(this) +
              sizeof(Frame) + count * sizeof(Entry))),
          stack({reinterpret_cast<Entry *>(reinterpret_cast<std::byte *>(this) + sizeof(Frame)), count}) {
        auto *entries = reinterpret_cast<Entry *>(reinterpret_cast<std::byte *>(this) + sizeof(Frame));
        for (std::uint64_t i = 0; i < count; ++i) {
            std::construct_at(entries + i);
            std::construct_at(records + i);
        }
    }

    static Panic drop(void *data) noexcept {
        const auto &record = *static_cast<Record *>(data);
        Panic panic;
        record.drop(record.data, &panic);
        return panic;
    }

    const std::uint64_t capacity;
    Record *const records;
    Stack stack;
    bool busy = false;
};

static_assert(sizeof(void *) == 8 && sizeof(std::size_t) == 8);
static_assert(sizeof(MeowyCleanupToken) == 24 && alignof(MeowyCleanupToken) == 8);
static_assert(sizeof(MeowyCleanupMark) == 24 && alignof(MeowyCleanupMark) == 8);
static_assert(offsetof(MeowyCleanupToken, index) == 8 && offsetof(MeowyCleanupToken, id) == 16);
static_assert(offsetof(MeowyCleanupMark, depth) == 8 && offsetof(MeowyCleanupMark, anchor) == 16);
static_assert(std::is_standard_layout_v<MeowyCleanupToken> && std::is_standard_layout_v<MeowyCleanupMark>);
static_assert(alignof(Frame) >= alignof(Entry) && alignof(Frame) >= alignof(Record));
static_assert(sizeof(Frame) % alignof(Entry) == 0 && sizeof(Entry) % alignof(Record) == 0);
static_assert(alignof(Frame) >= alignof(Panic) && alignof(Frame) <= 16);
static_assert(std::is_trivially_destructible_v<Entry> && std::is_trivially_destructible_v<Record>);
static_assert(std::is_trivially_destructible_v<Panic>);

bool aligned(const void *data, std::size_t alignment) noexcept {
    return data != nullptr && reinterpret_cast<std::uintptr_t>(data) % alignment == 0;
}

Frame *frame(void *data) noexcept {
    if (!aligned(data, alignof(Frame))) {
        return nullptr;
    }
    auto *value = static_cast<Frame *>(data);
    return value->busy ? nullptr : value;
}

std::uint32_t status(Status value) noexcept {
    return static_cast<std::uint32_t>(value);
}
}

extern "C" std::uint64_t meowy_cleanup_bytes_v0(std::uint64_t capacity) noexcept {
    constexpr auto limit = (std::numeric_limits<std::uint64_t>::max() - sizeof(Frame)) /
                           (sizeof(Entry) + sizeof(Record));
    return capacity > limit ? 0 : sizeof(Frame) + capacity * (sizeof(Entry) + sizeof(Record));
}

extern "C" std::uint64_t meowy_cleanup_alignment_v0() noexcept { return alignof(Frame); }

extern "C" std::uint32_t meowy_cleanup_open_v0(void *storage, std::uint64_t bytes,
                                               std::uint64_t capacity) noexcept {
    const auto required = meowy_cleanup_bytes_v0(capacity);
    if (!aligned(storage, alignof(Frame)) || required == 0 || bytes < required) {
        return status(Status::invalid);
    }
    std::construct_at(static_cast<Frame *>(storage), capacity);
    return status(Status::ok);
}

extern "C" std::uint32_t meowy_cleanup_mark_v0(void *data, MeowyCleanupMark *output) noexcept {
    const auto *value = frame(data);
    if (value == nullptr || output == nullptr) { return status(Status::invalid); }
    const auto mark = value->stack.mark();
    *output = {mark.stack, mark.depth, mark.anchor};
    return status(Status::ok);
}

extern "C" std::uint32_t meowy_cleanup_reserve_v0(void *data, MeowyCleanupToken *output) noexcept {
    auto *value = frame(data);
    if (value == nullptr || output == nullptr) { return status(Status::invalid); }
    const auto slot = value->stack.reserve();
    if (slot.status == Status::ok) {
        *output = {slot.token.stack, slot.token.index, slot.token.id};
    }
    return status(slot.status);
}

extern "C" std::uint32_t meowy_cleanup_arm_v0(void *data, const MeowyCleanupToken *token,
                                              void *payload, MeowyCleanupDrop drop,
                                              const char *name, std::uint64_t size) noexcept {
    auto *value = frame(data);
    if (value == nullptr || token == nullptr || token->index >= value->capacity ||
        drop == nullptr || name == nullptr || size == 0) { return status(Status::invalid); }
    auto &record = value->records[token->index];
    const auto result = value->stack.arm({static_cast<const Stack *>(token->owner), token->index, token->id},
                                          &record, Frame::drop, {name, size});
    if (result == Status::ok) { record = {payload, drop}; }
    return status(result);
}

extern "C" std::uint32_t meowy_cleanup_disarm_v0(void *data, const MeowyCleanupToken *token) noexcept {
    auto *value = frame(data);
    if (value == nullptr || token == nullptr) { return status(Status::invalid); }
    return status(value->stack.disarm({static_cast<const Stack *>(token->owner), token->index, token->id}));
}

extern "C" std::uint32_t meowy_cleanup_unwind_v0(void *data, const MeowyCleanupMark *mark,
                                                 std::uint32_t reason, const void *panic) noexcept {
    auto *value = frame(data);
    if (value == nullptr || mark == nullptr || reason > static_cast<std::uint32_t>(Reason::cancel) ||
        (reason == static_cast<std::uint32_t>(Reason::panic) && panic == nullptr) ||
        (panic != nullptr && !aligned(panic, alignof(Panic)))) { return status(Status::invalid); }
    const auto first = panic == nullptr ? Panic{} : *static_cast<const Panic *>(panic);
    value->busy = true;
    const auto result = value->stack.unwind({static_cast<const Stack *>(mark->owner), mark->depth, mark->anchor},
                                            static_cast<Reason>(reason), first);
    value->busy = false;
    return status(result.status);
}

extern "C" std::uint32_t meowy_cleanup_finish_v0(void *data) noexcept {
    auto *value = frame(data);
    if (value == nullptr || value->stack.size() != 0) { return status(Status::invalid); }
    std::destroy_at(value);
    return status(Status::ok);
}

extern "C" std::uint64_t meowy_cleanup_panic_bytes_v0() noexcept { return sizeof(Panic); }

extern "C" std::uint32_t meowy_cleanup_panic_init_v0(void *storage, std::uint64_t bytes,
                                                     std::uint32_t code, const char *text,
                                                     std::uint64_t size) noexcept {
    if (!aligned(storage, alignof(Panic)) || bytes < sizeof(Panic) || (text == nullptr && size != 0)) {
        return status(Status::invalid);
    }
    const Panic snapshot(code, {text == nullptr ? "" : text, size});
    std::construct_at(static_cast<Panic *>(storage), snapshot);
    return status(Status::ok);
}
