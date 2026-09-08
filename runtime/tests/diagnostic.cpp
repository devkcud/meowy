#include "meowy/cleanup.hpp"
#include "meowy/scheduler.hpp"

#include <algorithm>
#include <array>
#include <cstdio>
#include <cstdlib>
#include <limits>
#include <source_location>
#include <string_view>
#include <sys/resource.h>
#include <utility>

using namespace meowy::prototype::v0;

namespace {

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

void defaults_and_embedded_bytes() {
    const Panic empty;
    check(empty.code == 0 && empty.message().empty());
    check(empty.original_size() == 0 && !empty.truncated());
    constexpr std::array bytes{'a', '\0', 'b', '\n'};
    const Panic value{std::numeric_limits<std::uint32_t>::max(), {bytes.data(), bytes.size()}};
    check(value.code == std::numeric_limits<std::uint32_t>::max());
    check(value.message() == std::string_view(bytes.data(), bytes.size()));
    check(value.original_size() == bytes.size() && !value.truncated());
}

Panic local_message() noexcept {
    std::array text{'l', 'o', 'c', 'a', 'l'};
    const Panic result{6, {text.data(), text.size()}};
    text.fill('x');
    return result;
}

void constructor_captures_live_local_text() {
    const Panic value = local_message();
    check(value.code == 6 && value.message() == "local");
    check(value.original_size() == 5 && !value.truncated());
}

void copies_moves_and_replacement_are_independent() {
    Panic first{6, "original"};
    const Panic copy = first;
    first = Panic{2, "replacement"};
    Panic moved = std::move(first);
    first = Panic{9, "changed"};
    check(copy.code == 6 && copy.message() == "original");
    check(moved.code == 2 && moved.message() == "replacement");
    moved = Panic{moved.code, moved.message().substr(2)};
    check(moved.message() == "placement" && moved.original_size() == 9);
    check(copy.message() == "original" && first.message() == "changed");
}

void ascii_capacity_and_original_length_are_explicit() {
    constexpr auto cap = Panic::message_capacity;
    std::array<char, cap * 4> text;
    text.fill('a');
    for (const auto size : {std::size_t{0}, std::size_t{1}, cap - 1, cap, cap + 1, text.size()}) {
        const Panic value{7, {text.data(), size}};
        check(value.message() == std::string_view(text.data(), std::min(size, cap)));
        check(value.original_size() == size && value.truncated() == (size > cap));
        check(value.code == 7);
    }
}

void utf8_capacity_preserves_complete_codepoints() {
    constexpr auto cap = Panic::message_capacity;
    for (const std::string_view codepoint : {"\xc3\xa9", "\xe2\x82\xac", "\xf0\x9f\x90\xb1"}) {
        for (std::size_t offset = 0; offset <= codepoint.size(); ++offset) {
            std::array<char, cap + 5> text;
            text.fill('a');
            const auto start = cap - offset;
            std::copy(codepoint.begin(), codepoint.end(), text.begin() + start);
            const auto size = start + codepoint.size() + 1;
            text[size - 1] = 'z';
            const auto kept = offset == 0 || offset == codepoint.size() ? cap : start;
            const Panic value{6, {text.data(), size}};
            check(value.message() == std::string_view(text.data(), kept));
            check(value.original_size() == size && value.truncated());
        }
    }
}

void copied_truncation_metadata_survives_source_reuse() {
    constexpr auto cap = Panic::message_capacity;
    std::array<char, cap + 40> text;
    text.fill('b');
    Panic first{6, {text.data(), text.size()}};
    const Panic copy = first;
    text.fill('x');
    first = Panic{};
    const std::array<Panic, 2> stored{copy, Panic{2, "other"}};
    check(copy.code == 6 && copy.message().size() == cap);
    check(copy.message().find_first_not_of('b') == std::string_view::npos);
    check(copy.truncated() && copy.original_size() == cap + 40);
    check(stored[0].message() == copy.message() && stored[0].original_size() == cap + 40);
    check(stored[1].message() == "other" && !stored[1].truncated());
    check(first.code == 0 && first.message().empty() && !first.truncated());
}

void streamed_chunks_preserve_owned_prefix_and_utf8() {
    constexpr auto cap = Panic::message_capacity;
    for (const std::string_view point : {"a", "\xc3\xa9", "\xe2\x82\xac", "\xf0\x9f\x90\xb1"}) {
        for (std::size_t offset = 0; offset <= point.size(); ++offset) {
            std::array<char, cap + 8> text;
            text.fill('x');
            const auto start = cap - offset;
            std::copy(point.begin(), point.end(), text.begin() + start);
            const std::string_view input{text.data(), start + point.size() + 3};
            const Panic expected{6, input};
            for (std::size_t split = 0; split <= input.size(); ++split) {
                Panic value{6, {}};
                value.append(input.substr(0, split));
                value.append(input.substr(split));
                value.append({});
                check(value.message() == expected.message());
                check(value.original_size() == expected.original_size());
                check(value.truncated() == expected.truncated());
                const Panic copy = value;
                value.append("ignored tail");
                check(copy.message() == expected.message());
                check(value.message() == expected.message());
                check(value.original_size() == expected.original_size() + 12);
            }
        }
    }
    Panic value{6, "abc"};
    value.append(value.message());
    check(value.message() == "abcabc" && value.original_size() == 6);
    const Panic copy = value;
    value.append("def");
    check(copy.message() == "abcabc" && value.message() == "abcabcdef");
}

Panic truncated_cleanup(void *) noexcept {
    std::array<char, 260> text;
    text.fill('c');
    return {6, {text.data(), text.size()}};
}

[[noreturn]] void fatal_truncated() {
    std::array<Entry, 1> entries;
    Stack stack(entries);
    const auto mark = stack.mark();
    const auto slot = stack.reserve();
    check(slot.status == Status::ok);
    check(stack.arm(slot.token, nullptr, truncated_cleanup, "truncated messages") == Status::ok);
    std::array<char, 300> text;
    text.fill('b');
    const Panic first{6, {text.data(), text.size()}};
    text.fill('x');
    static_cast<void>(stack.unwind(mark, Reason::panic, first));
    std::abort();
}

struct Case final {
public:
    const char *name;
    void (*run)();
};

constexpr std::array cases{
    Case{"streamed_chunks_preserve_owned_prefix_and_utf8", streamed_chunks_preserve_owned_prefix_and_utf8},
    Case{"defaults_and_embedded_bytes", defaults_and_embedded_bytes},
    Case{"constructor_captures_live_local_text", constructor_captures_live_local_text},
    Case{"copies_moves_and_replacement_are_independent", copies_moves_and_replacement_are_independent},
    Case{"ascii_capacity_and_original_length_are_explicit", ascii_capacity_and_original_length_are_explicit},
    Case{"utf8_capacity_preserves_complete_codepoints", utf8_capacity_preserves_complete_codepoints},
    Case{"copied_truncation_metadata_survives_source_reuse", copied_truncation_metadata_survives_source_reuse},
};

}

int main(int argc, char **argv) {
    const rlimit limit{0, 0};
    check(setrlimit(RLIMIT_CORE, &limit) == 0);
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-truncated") {
        fatal_truncated();
    }
    if (argc == 2 && std::string_view(argv[1]) == "--layout") {
        std::printf("Panic=%zu TaskOutcome=%zu TaskSlot=%zu TaskInfo=%zu Joined=%zu ChildFailure=%zu ScopeClose=%zu\n",
                    sizeof(Panic), sizeof(TaskOutcome), sizeof(TaskSlot), sizeof(TaskInfo),
                    sizeof(Joined), sizeof(ChildFailure), sizeof(ScopeClose));
        return 0;
    }
    if (argc != 1) {
        return 2;
    }
    for (const auto &item : cases) {
        item.run();
        std::printf("PASS %s\n", item.name);
    }
    std::printf("%zu diagnostic snapshot cases passed\n", cases.size());
}
