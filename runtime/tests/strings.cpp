#include "meowy/strings.hpp"
#include "meowy/generated.hpp"

#include <array>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <new>
#include <source_location>

using namespace meowy::prototype::v0;

namespace {

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

struct Counts final {
public:
    std::size_t allocated = 0;
    std::size_t released = 0;
    std::size_t size = 0;
    bool fail = false;
    Owned *owner = nullptr;
};

void guard(Counts &counts) {
    if (counts.owner != nullptr) {
        check(counts.owner->data() == nullptr);
        check(counts.owner->release() == OwnedStatus::invalid);
        check(counts.owner->commit() == OwnedStatus::invalid);
        AllocationFailure failure;
        check(Strings::copy(*counts.owner, "nested", Strings::heap(), failure) == StringStatus::invalid);
        std::string_view view;
        check(!Strings::view(*counts.owner, view));
    }
}

void *allocate(void *context, std::size_t size) noexcept {
    auto &counts = *static_cast<Counts *>(context);
    guard(counts);
    ++counts.allocated;
    counts.size = size;
    return counts.fail ? nullptr : std::malloc(size);
}

void release(void *context, void *data, std::size_t size) noexcept {
    auto &counts = *static_cast<Counts *>(context);
    guard(counts);
    check(data != nullptr && size == counts.size);
    ++counts.released;
    std::free(data);
}

void copies_bytes_before_input_storage_changes() {
    Counts counts;
    const Allocator allocator{&counts, allocate, release};
    alignas(16) std::array<std::byte, 64> bytes;
    Owned owner(bytes);
    AllocationFailure failure;
    std::array text{'a', '\0', '\xc3', '\xa9'};
    check(Strings::copy(owner, {text.data(), text.size()}, allocator, failure) == StringStatus::ok);
    text.fill('x');
    std::string_view view;
    check(Strings::view(owner, view) && view == std::string_view("a\0\xc3\xa9", 4));
    check(failure.cause == AllocationCause::none && failure.bytes == 0);
    check(counts.allocated == 1 && counts.released == 0 && counts.size == 4);
    check(owner.release() == OwnedStatus::ok && owner.release() == OwnedStatus::ok);
    check(counts.released == 1 && owner.empty());
}

void empty_strings_have_live_ownership_without_allocation() {
    Counts counts;
    counts.fail = true;
    const Allocator allocator{&counts, allocate, release};
    alignas(16) std::array<std::byte, 64> bytes;
    Owned owner(bytes);
    AllocationFailure failure;
    check(Strings::copy(owner, {}, allocator, failure) == StringStatus::ok);
    check(owner.initialized() && counts.allocated == 0);
    std::string_view view = "old";
    check(Strings::view(owner, view) && view.empty());
    check(owner.release() == OwnedStatus::ok && counts.released == 0);
}

void allocation_failure_preserves_facts_and_allows_retry() {
    Counts counts;
    counts.fail = true;
    const Allocator allocator{&counts, allocate, release};
    alignas(16) std::array<std::byte, 64> bytes;
    Owned owner(bytes);
    AllocationFailure failure;
    check(Strings::copy(owner, "hello", allocator, failure) == StringStatus::allocation_failed);
    check(owner.empty() && counts.released == 0);
    check(failure.cause == AllocationCause::exhausted && failure.bytes == 5 && failure.alignment == 1);
    std::string_view view = "unchanged";
    check(!Strings::view(owner, view) && view == "unchanged");
    counts.fail = false;
    check(Strings::copy(owner, "hello", allocator, failure) == StringStatus::ok);
    check(failure.cause == AllocationCause::none && failure.bytes == 0 && failure.alignment == 0);
    check(owner.release() == OwnedStatus::ok && counts.allocated == 2 && counts.released == 1);
}

void storage_preflight_does_not_allocate_or_replace_live_values() {
    Counts counts;
    const Allocator allocator{&counts, allocate, release};
    alignas(16) std::array<std::byte, 128> bytes;
    Owned short_owner({bytes.data(), 1});
    Owned misaligned({bytes.data() + 1, 64});
    Owned owner({bytes.data() + 64, 64});
    AllocationFailure failure{AllocationCause::exhausted, 99, 9};
    check(Strings::copy(short_owner, "x", allocator, failure) == StringStatus::full);
    check(Strings::copy(misaligned, "x", allocator, failure) == StringStatus::misaligned);
    check(counts.allocated == 0 && failure.bytes == 99);
    check(Strings::copy(owner, "first", allocator, failure) == StringStatus::ok);
    check(Strings::copy(owner, "second", allocator, failure) == StringStatus::occupied);
    std::string_view view;
    check(Strings::view(owner, view) && view == "first" && counts.allocated == 1);
    check(owner.release() == OwnedStatus::ok && counts.released == 1);
    const Allocator invalid{nullptr, nullptr, nullptr};
    check(Strings::copy(owner, "x", invalid, failure) == StringStatus::invalid);
    check(owner.empty());
}

void moves_transfer_one_release_and_callbacks_reject_reentry() {
    Counts counts;
    const Allocator allocator{&counts, allocate, release};
    alignas(16) std::array<std::byte, 64> first;
    alignas(16) std::array<std::byte, 64> second;
    Owned a(first);
    Owned b(second);
    counts.owner = &a;
    AllocationFailure failure;
    check(Strings::copy(a, "hello", allocator, failure) == StringStatus::ok);
    std::string_view before;
    check(Strings::view(a, before));
    Owned short_owner;
    check(a.move_to(short_owner) == OwnedStatus::full);
    check(a.initialized() && counts.released == 0);
    check(a.move_to(b) == OwnedStatus::ok && a.empty());
    std::string_view after;
    check(Strings::view(b, after) && after == "hello" && after.data() == before.data());
    check(!Strings::view(a, after));
    counts.owner = &b;
    check(b.release() == OwnedStatus::ok && counts.allocated == 1 && counts.released == 1);
}

void generated_bridge_transfers_strings_and_unwinds_once() {
    Counts counts;
    const Allocator allocator{&counts, allocate, release};
    alignas(16) std::array<std::byte, 64> first;
    alignas(16) std::array<std::byte, 64> second;
    Owned a(first);
    Owned b(second);
    AllocationFailure failure;
    check(meowy_string_copy_v0(&a, "text", 4, &allocator, &failure) == 0);
    alignas(16) std::array<std::byte, 4096> frame;
    const auto size = meowy_cleanup_bytes_v0(2);
    check(size <= frame.size());
    check(meowy_cleanup_open_v0(frame.data(), frame.size(), 2) == 0);
    MeowyCleanupMark mark;
    MeowyCleanupToken x;
    MeowyCleanupToken y;
    check(meowy_cleanup_mark_v0(frame.data(), &mark) == 0);
    check(meowy_cleanup_reserve_v0(frame.data(), &x) == 0);
    check(meowy_owned_arm_v0(frame.data(), &x, &a) == 0);
    check(meowy_cleanup_reserve_v0(frame.data(), &y) == 0);
    check(meowy_owned_transfer_v0(frame.data(), &x, &a, frame.data(), &y, &b) == 0);
    const char *text = nullptr;
    std::uint64_t length = 0;
    check(meowy_string_view_v0(&b, &text, &length) == 0);
    check(std::string_view(text, length) == "text");
    const Panic panic{6, "original"};
    check(meowy_cleanup_unwind_v0(frame.data(), &mark, 3, &panic) == 0);
    check(meowy_cleanup_finish_v0(frame.data()) == 0);
    check(a.empty() && b.empty() && counts.allocated == 1 && counts.released == 1);
}

void heap_and_scalar_bridge_validate_storage_and_outputs() {
    static_assert(sizeof(AllocationFailure) == 24 && alignof(AllocationFailure) == 8);
    check(meowy_string_heap_v0() == &Strings::heap());
    check(meowy_string_ops_v0() == &Strings::ops());
    check(meowy_string_bytes_v0() <= 64 && meowy_string_alignment_v0() <= 16);
    alignas(16) std::array<std::byte, 64> bytes;
    Owned owner(bytes);
    AllocationFailure failure;
    const char *text = "kept";
    std::uint64_t size = 4;
    check(meowy_string_view_v0(&owner, &text, &size) == 1);
    check(std::string_view(text, size) == "kept");
    static const ValueOps scalar{
        sizeof(std::uint64_t), alignof(std::uint64_t),
        [](void *destination, void *source) noexcept {
            new (destination) std::uint64_t{*static_cast<std::uint64_t *>(source)};
        },
        [](void *) noexcept -> Panic { return {}; }, "scalar"
    };
    check(owner.reserve(scalar) == OwnedStatus::ok);
    new (owner.data()) std::uint64_t{7};
    check(owner.commit() == OwnedStatus::ok);
    check(meowy_string_view_v0(&owner, &text, &size) == 1);
    check(std::string_view(text, size) == "kept");
    check(owner.release() == OwnedStatus::ok);
    check(meowy_string_copy_v0(nullptr, "x", 1, meowy_string_heap_v0(), &failure) == 1);
    check(meowy_string_copy_v0(&owner, nullptr, 1, meowy_string_heap_v0(), &failure) == 1);
    check(meowy_string_copy_v0(&owner, "x", 1, nullptr, &failure) == 1);
    check(meowy_string_copy_v0(&owner, "x", 1, meowy_string_heap_v0(), nullptr) == 1);
    check(meowy_string_copy_v0(&owner, "heap", 4, meowy_string_heap_v0(), &failure) == 0);
    check(meowy_string_view_v0(&owner, nullptr, &size) == 1);
    check(meowy_string_view_v0(&owner, &text, nullptr) == 1);
    check(meowy_string_view_v0(&owner, &text, &size) == 0 && std::string_view(text, size) == "heap");
    check(owner.release() == OwnedStatus::ok);
}

struct Case final {
public:
    const char *name;
    void (*run)();
};

constexpr std::array cases{
    Case{"copies_bytes_before_input_storage_changes", copies_bytes_before_input_storage_changes},
    Case{"empty_strings_have_live_ownership_without_allocation", empty_strings_have_live_ownership_without_allocation},
    Case{"allocation_failure_preserves_facts_and_allows_retry", allocation_failure_preserves_facts_and_allows_retry},
    Case{"storage_preflight_does_not_allocate_or_replace_live_values", storage_preflight_does_not_allocate_or_replace_live_values},
    Case{"moves_transfer_one_release_and_callbacks_reject_reentry", moves_transfer_one_release_and_callbacks_reject_reentry},
    Case{"generated_bridge_transfers_strings_and_unwinds_once", generated_bridge_transfers_strings_and_unwinds_once},
    Case{"heap_and_scalar_bridge_validate_storage_and_outputs", heap_and_scalar_bridge_validate_storage_and_outputs},
};

}

int main() {
    for (const auto &item : cases) {
        item.run();
        std::printf("PASS %s\n", item.name);
    }
    std::printf("%zu owned string cases passed\n", cases.size());
}
