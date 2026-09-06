#include "meowy/owned.hpp"
#include "meowy/scheduler.hpp"

#include <array>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <fcntl.h>
#include <memory>
#include <source_location>
#include <string_view>
#include <sys/mman.h>
#include <sys/resource.h>
#include <unistd.h>

using namespace meowy::prototype::v0;

namespace {

constexpr std::size_t stack_bytes = 262144;
bool fail_map = false;
bool fail_protect = false;
bool fail_release = false;

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

struct alignas(64) Buffer final {
public:
    std::array<std::byte, 128> bytes{};
};

struct Log final {
public:
    std::array<bool, 16> live{};
    std::array<int, 32> drops{};
    std::size_t count = 0;
    int moves = 0;
    int calls = 0;
    Scheduler *scheduler = nullptr;
    Task *task = nullptr;
    Owned *source = nullptr;
    Owned *destination = nullptr;
};

Panic empty(Task &, void *) noexcept { return {}; }

struct Resource final {
public:
    Log *log;
    int id;
    int value = 41;
    int fd = -1;
    bool fail = false;

    static void move(void *destination, void *source) noexcept {
        auto &value = *static_cast<Resource *>(source);
        auto &log = *value.log;
        check(log.live[value.id]);
        if (log.source != nullptr) { check(log.source->release() == OwnedStatus::invalid); }
        if (log.destination != nullptr) { check(log.destination->release() == OwnedStatus::invalid); }
        if (log.scheduler != nullptr) { check(log.scheduler->submit(empty, nullptr).status == ScheduleStatus::invalid); }
        if (log.task != nullptr) {
            check(log.task->yield() == ContextStatus::invalid);
            check(log.task->mark().status == ScheduleStatus::invalid);
            check(log.task->close({}).status == ScheduleStatus::invalid);
        }
        std::construct_at(static_cast<Resource *>(destination), value);
        ++log.moves;
        std::destroy_at(&value);
    }

    static Panic drop(void *data) noexcept {
        auto &value = *static_cast<Resource *>(data);
        auto &log = *value.log;
        check(log.live[value.id]);
        check(log.count < log.drops.size());
        if (log.scheduler != nullptr) { check(log.scheduler->submit(empty, nullptr).status == ScheduleStatus::invalid); }
        if (log.task != nullptr) {
            check(log.task->yield() == ContextStatus::invalid);
            check(log.task->mark().status == ScheduleStatus::invalid);
            check(log.task->close({}).status == ScheduleStatus::invalid);
        }
        log.task = nullptr;
        log.live[value.id] = false;
        log.drops[log.count++] = value.id;
        if (value.fd >= 0) { check(close(value.fd) == 0); }
        const bool fail = value.fail;
        std::destroy_at(&value);
        if (fail) {
            constexpr std::string_view text = "release-trigger\n";
            check(::write(STDERR_FILENO, text.data(), text.size()) == static_cast<ssize_t>(text.size()));
            return {6, "release failed"};
        }
        return {};
    }
};

constexpr ValueOps ops{sizeof(Resource), alignof(Resource), Resource::move, Resource::drop, "owned resource"};

void initialize(Owned &owner, Log &log, int id) {
    check(owner.reserve(ops) == OwnedStatus::ok);
    check(!log.live[id]);
    std::construct_at(static_cast<Resource *>(owner.data()), Resource{&log, id});
    log.live[id] = true;
    check(owner.commit() == OwnedStatus::ok);
}

struct Fixture final {
public:
    Buffer capture;
    Buffer result;
    std::array<TaskSlot, 1> slots{TaskSlot(capture.bytes, result.bytes)};
    Scheduler scheduler{slots, stack_bytes};
};

Panic consume(Task &task, void *data) noexcept {
    auto &value = *static_cast<Resource *>(data);
    check(value.log->live[value.id]);
    ++value.log->calls;
    ++value.value;
    if (value.log->scheduler != nullptr) { value.log->task = &task; }
    return {};
}

Panic forward(Task &task, void *data) noexcept {
    auto &value = *static_cast<Resource *>(data);
    check(value.log->live[value.id]);
    ++value.log->calls;
    ++value.value;
    check(task.emit_capture() == OwnedStatus::ok);
    return {};
}

Panic produce(Task &task, void *data) noexcept {
    auto &capture = *static_cast<Resource *>(data);
    Buffer buffer;
    Owned result(buffer.bytes);
    initialize(result, *capture.log, 2);
    check(task.set_result(result) == OwnedStatus::ok && result.empty());
    return {};
}

Panic produce_then_panic(Task &task, void *data) noexcept {
    check(produce(task, data).code == 0);
    return {6, "body failed"};
}

Panic forward_then_panic(Task &task, void *data) noexcept {
    check(forward(task, data).code == 0);
    return {6, "body failed"};
}

void reserved_storage_is_not_destroyed_before_commit() {
    Buffer buffer;
    Owned owner(buffer.bytes);
    check(owner.reserve(ops) == OwnedStatus::ok);
    check(!owner.initialized() && !owner.empty());
    check(owner.release() == OwnedStatus::ok && owner.empty());
    Log log;
    initialize(owner, log, 1);
    check(owner.commit() == OwnedStatus::invalid);
    check(owner.reserve(ops) == OwnedStatus::occupied);
    check(owner.release() == OwnedStatus::ok);
    check(owner.release() == OwnedStatus::ok && log.count == 1);
}

void relocation_checks_capacity_alignment_overlap_and_reentry() {
    Buffer input;
    Buffer output;
    Log log;
    Owned source(input.bytes);
    Owned destination(output.bytes);
    Owned tiny(std::span(output.bytes).first(1));
    Owned unaligned(std::span(output.bytes).subspan(1));
    Owned overlap(input.bytes);
    initialize(source, log, 1);
    check(source.move_to(tiny) == OwnedStatus::full);
    check(source.move_to(unaligned) == OwnedStatus::misaligned);
    check(source.move_to(overlap) == OwnedStatus::overlap);
    check(source.move_to(source) == OwnedStatus::invalid);
    log.source = &source;
    log.destination = &destination;
    check(source.move_to(destination) == OwnedStatus::ok);
    log.source = nullptr;
    log.destination = nullptr;
    check(source.empty() && destination.initialized() && destination.data() != input.bytes.data());
    check(log.moves == 1 && log.count == 0);
    check(destination.release() == OwnedStatus::ok && log.count == 1);
}

void accepted_capture_is_relocated_and_released_once() {
    Fixture fixture;
    Buffer buffer;
    Log log;
    Owned capture(buffer.bytes);
    initialize(capture, log, 1);
    log.scheduler = &fixture.scheduler;
    const auto submitted = fixture.scheduler.submit_owned(consume, capture);
    check(submitted.status == ScheduleStatus::ok && capture.empty());
    check(log.moves == 1 && log.count == 0 && log.calls == 0);
    check(fixture.scheduler.pump(1).resumed == 1);
    check(log.calls == 1 && log.count == 1 && !log.live[1]);
    check(fixture.scheduler.join(submitted.ticket).status == ScheduleStatus::ok && log.count == 1);
}

void invalid_submission_preserves_but_full_submission_drops_capture() {
    Fixture fixture;
    Buffer buffer;
    Log log;
    Owned capture(buffer.bytes);
    initialize(capture, log, 1);
    check(fixture.scheduler.submit_owned(nullptr, capture).status == ScheduleStatus::invalid);
    check(capture.initialized() && log.count == 0);
    const auto first = fixture.scheduler.submit(empty, nullptr);
    check(first.status == ScheduleStatus::ok);
    const auto failed = fixture.scheduler.submit_owned(consume, capture);
    check(failed.status == ScheduleStatus::full && failed.ticket.scheduler == nullptr);
    check(capture.empty() && log.count == 1 && log.moves == 0 && log.calls == 0);
    check(fixture.scheduler.pump(1).resumed == 1);
    check(fixture.scheduler.join(first.ticket).status == ScheduleStatus::ok);
}

void storage_and_os_admission_failure_release_accepted_captures() {
    for (int mode = 0; mode < 3; ++mode) {
        Buffer input;
        Buffer output;
        Buffer source;
        const auto capacity = mode == 0 ? 1 : input.bytes.size();
        std::array<TaskSlot, 1> slots{TaskSlot(std::span(input.bytes).first(capacity), output.bytes)};
        Scheduler scheduler(slots, stack_bytes);
        Log log;
        Owned capture(source.bytes);
        initialize(capture, log, 1);
        fail_map = mode == 1;
        fail_protect = mode == 2;
        fail_release = mode == 2;
        const auto submitted = scheduler.submit_owned(consume, capture);
        fail_map = false;
        fail_protect = false;
        check(submitted.status == (mode == 0 ? ScheduleStatus::storage_failed : ScheduleStatus::context_failed));
        check(capture.empty() && log.count == 1 && log.calls == 0);
        check(log.moves == (mode == 0 ? 0 : 1));
        if (mode == 2) { check(scheduler.join(submitted.ticket).status == ScheduleStatus::context_failed); }
        fail_release = false;
        const auto joined = scheduler.join(submitted.ticket);
        check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::spawn_failed);
        check(log.count == 1);
    }
}

void owned_result_survives_bad_destination_and_join_retry() {
    Fixture fixture;
    Buffer input;
    Buffer output;
    Log log;
    Owned capture(input.bytes);
    Owned result(output.bytes);
    Owned tiny(std::span(output.bytes).first(1));
    initialize(capture, log, 1);
    const auto submitted = fixture.scheduler.submit_owned(forward, capture);
    check(submitted.status == ScheduleStatus::ok && fixture.scheduler.pump(1).resumed == 1);
    check(log.moves == 2 && log.count == 0 && fixture.scheduler.inspect(submitted.ticket).has_result);
    check(fixture.scheduler.join(submitted.ticket).status == ScheduleStatus::storage_failed);
    check(fixture.scheduler.join_owned(submitted.ticket, tiny).storage == OwnedStatus::full);
    Log occupied;
    initialize(result, occupied, 3);
    check(fixture.scheduler.join_owned(submitted.ticket, result).storage == OwnedStatus::occupied);
    check(result.initialized() && occupied.count == 0 && log.count == 0);
    check(result.release() == OwnedStatus::ok && occupied.count == 1);
    fail_release = true;
    check(fixture.scheduler.join_owned(submitted.ticket, result).status == ScheduleStatus::context_failed);
    fail_release = false;
    check(result.empty() && log.moves == 2 && log.count == 0);
    check(fixture.scheduler.inspect(submitted.ticket).has_result);
    const auto joined = fixture.scheduler.join_owned(submitted.ticket, result);
    check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::completed);
    check(result.initialized() && log.moves == 3 && log.count == 0);
    check(static_cast<Resource *>(result.data())->value == 42);
    check(fixture.scheduler.join_owned(submitted.ticket, result).status == ScheduleStatus::invalid);
    check(result.release() == OwnedStatus::ok && log.count == 1);
}

void normal_and_panicked_results_have_exact_cleanup_order() {
    for (const auto body : {produce, produce_then_panic, forward_then_panic}) {
        Fixture fixture;
        Buffer input;
        Buffer output;
        Log log;
        Owned capture(input.bytes);
        Owned result(output.bytes);
        initialize(capture, log, 1);
        const auto submitted = fixture.scheduler.submit_owned(body, capture);
        check(submitted.status == ScheduleStatus::ok && fixture.scheduler.pump(1).resumed == 1);
        if (body == produce) {
            check(log.count == 1 && log.drops[0] == 1 && log.live[2]);
            check(fixture.scheduler.join_owned(submitted.ticket, result).status == ScheduleStatus::ok);
            check(result.release() == OwnedStatus::ok && log.count == 2 && log.drops[1] == 2);
        } else {
            check(!fixture.scheduler.inspect(submitted.ticket).has_result);
            const auto joined = fixture.scheduler.join(submitted.ticket);
            check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::panicked);
            check(joined.outcome.panic.message == "body failed");
            if (body == produce_then_panic) { check(log.count == 2 && log.drops[0] == 2 && log.drops[1] == 1); }
            else { check(log.count == 1 && log.drops[0] == 1); }
        }
    }
}

Panic rejected_result(Task &task, void *data) noexcept {
    auto &capture = *static_cast<Resource *>(data);
    Buffer buffer;
    Owned value(buffer.bytes);
    initialize(value, *capture.log, 2);
    check(task.set_result(value) == OwnedStatus::full && value.initialized());
    check(value.release() == OwnedStatus::ok);
    return {};
}

void failed_result_transfer_retains_source_ownership() {
    Buffer input;
    Buffer source;
    std::array<TaskSlot, 1> slots{TaskSlot(input.bytes, {})};
    Scheduler scheduler(slots, stack_bytes);
    Log log;
    Owned capture(source.bytes);
    initialize(capture, log, 1);
    const auto submitted = scheduler.submit_owned(rejected_result, capture);
    check(submitted.status == ScheduleStatus::ok && scheduler.pump(1).resumed == 1);
    check(log.count == 2 && log.drops[0] == 2 && log.drops[1] == 1);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::ok);
}

Panic parent_owned(Task &task, void *data) noexcept {
    auto &log = *static_cast<Log *>(data);
    Buffer input;
    Buffer output;
    Owned capture(input.bytes);
    Owned result(output.bytes);
    initialize(capture, log, 1);
    const auto child = task.spawn_owned(forward, capture);
    check(child.status == ScheduleStatus::ok && capture.empty());
    fail_release = true;
    check(task.join_owned(child.ticket, result).status == ScheduleStatus::context_failed);
    fail_release = false;
    check(result.empty() && log.count == 0);
    check(task.join_owned(child.ticket, result).status == ScheduleStatus::ok);
    check(task.set_result(result) == OwnedStatus::ok && result.empty());
    return {};
}

void owned_child_results_transfer_through_parent_after_join_retry() {
    std::array<Buffer, 2> input;
    std::array<Buffer, 2> output;
    std::array<TaskSlot, 2> slots{TaskSlot(input[0].bytes, output[0].bytes), TaskSlot(input[1].bytes, output[1].bytes)};
    Scheduler scheduler(slots, stack_bytes);
    Log log;
    Buffer destination;
    Owned result(destination.bytes);
    const auto parent = scheduler.submit(parent_owned, &log);
    check(parent.status == ScheduleStatus::ok && scheduler.pump(10).status == ScheduleStatus::ok);
    check(log.count == 0);
    check(scheduler.join_owned(parent.ticket, result).status == ScheduleStatus::ok);
    check(result.release() == OwnedStatus::ok && log.count == 1);
}

Panic borrow_capture(Task &task, void *data) noexcept {
    auto &value = *static_cast<Resource *>(data);
    check(value.log->live[value.id]);
    check(task.yield() == ContextStatus::ok);
    ++value.value;
    return {};
}

Panic parent_borrow(Task &task, void *data) noexcept {
    const auto child = task.spawn(borrow_capture, data);
    check(child.status == ScheduleStatus::ok);
    check(task.emit_capture() == OwnedStatus::invalid);
    check(task.join(child.ticket).status == ScheduleStatus::ok);
    check(task.emit_capture() == OwnedStatus::ok);
    return {};
}

void capture_storage_cannot_move_until_borrowing_children_join() {
    std::array<Buffer, 2> input;
    std::array<Buffer, 2> output;
    std::array<TaskSlot, 2> slots{TaskSlot(input[0].bytes, output[0].bytes), TaskSlot(input[1].bytes, output[1].bytes)};
    Scheduler scheduler(slots, stack_bytes);
    Buffer source;
    Buffer destination;
    Owned capture(source.bytes);
    Owned result(destination.bytes);
    Log log;
    initialize(capture, log, 1);
    const auto parent = scheduler.submit_owned(parent_borrow, capture);
    check(parent.status == ScheduleStatus::ok && scheduler.pump(10).status == ScheduleStatus::ok);
    check(scheduler.join_owned(parent.ticket, result).status == ScheduleStatus::ok);
    check(static_cast<Resource *>(result.data())->value == 42);
    check(result.release() == OwnedStatus::ok && log.count == 1);
}

void transferred_file_descriptor_closes_only_at_final_owner_release() {
    Fixture fixture;
    Buffer input;
    Buffer output;
    Log log;
    Owned capture(input.bytes);
    Owned result(output.bytes);
    std::array<int, 2> descriptors{};
    check(pipe(descriptors.data()) == 0);
    initialize(capture, log, 1);
    static_cast<Resource *>(capture.data())->fd = descriptors[1];
    const auto submitted = fixture.scheduler.submit_owned(forward, capture);
    check(submitted.status == ScheduleStatus::ok && fixture.scheduler.pump(1).resumed == 1);
    check(fixture.scheduler.join_owned(submitted.ticket, result).status == ScheduleStatus::ok);
    check(fcntl(descriptors[1], F_GETFD) >= 0 && log.count == 0);
    check(result.release() == OwnedStatus::ok && log.count == 1);
    errno = 0;
    check(fcntl(descriptors[1], F_GETFD) == -1 && errno == EBADF);
    char value = 0;
    check(read(descriptors[0], &value, 1) == 0);
    check(close(descriptors[0]) == 0);
}

Panic emit_during_cleanup(Task &task, void *data) noexcept {
    Buffer buffer;
    Owned result(buffer.bytes);
    initialize(result, *static_cast<Log *>(data), 2);
    check(task.set_result(result) == OwnedStatus::invalid && result.initialized());
    check(result.release() == OwnedStatus::ok);
    return {};
}

void cleanup_cannot_emit_an_owned_result() {
    Fixture fixture;
    Log log;
    const auto submitted = fixture.scheduler.submit(empty, &log, emit_during_cleanup);
    check(submitted.status == ScheduleStatus::ok && fixture.scheduler.pump(1).resumed == 1);
    check(log.count == 1 && !fixture.scheduler.inspect(submitted.ticket).has_result);
    check(fixture.scheduler.join(submitted.ticket).status == ScheduleStatus::ok);
}

struct ScopeRetry final {
public:
    Log log;
    ScopeClose first;
    bool paused = false;
    bool done = false;

    static Panic failed_child(Task &, void *) noexcept { return {6, "first child failed"}; }

    static Panic held_result(Task &task, void *) noexcept {
        check(task.yield() == ContextStatus::ok);
        fail_release = true;
        check(task.emit_capture() == OwnedStatus::ok);
        return {};
    }

    static Panic parent(Task &task, void *data) noexcept {
        auto &value = *static_cast<ScopeRetry *>(data);
        const auto opened = task.mark();
        check(opened.status == ScheduleStatus::ok);
        const auto copy = opened.mark;
        check(task.spawn(failed_child, nullptr).status == ScheduleStatus::ok);
        Buffer buffer;
        Owned capture(buffer.bytes);
        initialize(capture, value.log, 1);
        const auto child = task.spawn_owned(held_result, capture);
        check(child.status == ScheduleStatus::ok && capture.empty());
        std::array<ChildFailure, 1> failures;
        value.first = task.close(opened.mark, failures);
        fail_release = false;
        check(value.first.status == ScheduleStatus::context_failed && value.first.joined == 1 && value.first.panicked == 1);
        check(value.first.reported == 1 && failures[0].outcome.panic.message == "first child failed");
        check(value.first.first_failure.panic.message == "first child failed");
        check(value.first.pending == child.ticket && value.first.pending_result.context.memory.error == EIO);
        check(value.first.pending_result.outcome.kind == OutcomeKind::completed && value.log.count == 0);
        value.paused = true;
        check(task.yield() == ContextStatus::ok);
        value.log.task = &task;
        const auto closed = task.close(copy, failures);
        check(closed.status == ScheduleStatus::ok && closed.joined == 2 && closed.panicked == 1 && closed.spawn_failed == 0);
        check(closed.reported == 0 && failures[0].outcome.panic.message == "first child failed");
        check(closed.first_failure.panic.message == "first child failed" && closed.pending.scheduler == nullptr);
        check(value.log.count == 1 && !value.log.live[1]);
        check(task.close(opened.mark).status == ScheduleStatus::invalid);
        value.done = true;
        return {};
    }
};

void scope_close_preserves_progress_and_owned_result_until_release_succeeds() {
    std::array<Buffer, 3> input;
    std::array<Buffer, 3> output;
    std::array<TaskSlot, 3> slots{TaskSlot(input[0].bytes, output[0].bytes), TaskSlot(input[1].bytes, output[1].bytes),
                                TaskSlot(input[2].bytes, output[2].bytes)};
    Scheduler scheduler(slots, stack_bytes);
    ScopeRetry value;
    const auto parent = scheduler.submit(ScopeRetry::parent, &value);
    check(parent.status == ScheduleStatus::ok && scheduler.pump(6).resumed == 6);
    check(value.paused && !value.done && value.log.count == 0);
    const auto state = scheduler.inspect(parent.ticket);
    check(state.scopes == 1 && state.children == 1);
    check(scheduler.inspect(value.first.pending).has_result);
    check(scheduler.pump(2).status == ScheduleStatus::ok && value.done);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok && value.log.count == 1);
}

Panic scope_drop_failure(Task &task, void *data) noexcept {
    auto &log = *static_cast<Log *>(data);
    const auto scope = task.mark();
    check(scope.status == ScheduleStatus::ok);
    Buffer buffer;
    Owned capture(buffer.bytes);
    initialize(capture, log, 1);
    static_cast<Resource *>(capture.data())->fail = true;
    check(task.spawn_owned(forward, capture).status == ScheduleStatus::ok);
    static_cast<void>(task.close(scope.mark));
    return {};
}

struct Case final {
public:
    std::string_view name;
    void (*run)();
};

constexpr std::array cases{
    Case{"reserved_storage_is_not_destroyed_before_commit", reserved_storage_is_not_destroyed_before_commit},
    Case{"relocation_checks_capacity_alignment_overlap_and_reentry", relocation_checks_capacity_alignment_overlap_and_reentry},
    Case{"accepted_capture_is_relocated_and_released_once", accepted_capture_is_relocated_and_released_once},
    Case{"invalid_submission_preserves_but_full_submission_drops_capture", invalid_submission_preserves_but_full_submission_drops_capture},
    Case{"storage_and_os_admission_failure_release_accepted_captures", storage_and_os_admission_failure_release_accepted_captures},
    Case{"owned_result_survives_bad_destination_and_join_retry", owned_result_survives_bad_destination_and_join_retry},
    Case{"normal_and_panicked_results_have_exact_cleanup_order", normal_and_panicked_results_have_exact_cleanup_order},
    Case{"failed_result_transfer_retains_source_ownership", failed_result_transfer_retains_source_ownership},
    Case{"owned_child_results_transfer_through_parent_after_join_retry", owned_child_results_transfer_through_parent_after_join_retry},
    Case{"capture_storage_cannot_move_until_borrowing_children_join", capture_storage_cannot_move_until_borrowing_children_join},
    Case{"transferred_file_descriptor_closes_only_at_final_owner_release", transferred_file_descriptor_closes_only_at_final_owner_release},
    Case{"cleanup_cannot_emit_an_owned_result", cleanup_cannot_emit_an_owned_result},
    Case{"scope_close_preserves_progress_and_owned_result_until_release_succeeds", scope_close_preserves_progress_and_owned_result_until_release_succeeds},
};

}

extern "C" void *__real_mmap(void *, std::size_t, int, int, int, off_t);
extern "C" int __real_mprotect(void *, std::size_t, int);
extern "C" int __real_munmap(void *, std::size_t);

extern "C" void *__wrap_mmap(void *address, std::size_t size, int protection, int flags, int descriptor, off_t offset) {
    if (fail_map) { errno = ENOMEM; return MAP_FAILED; }
    return __real_mmap(address, size, protection, flags, descriptor, offset);
}

extern "C" int __wrap_mprotect(void *address, std::size_t size, int protection) {
    if (fail_protect) { errno = EACCES; return -1; }
    return __real_mprotect(address, size, protection);
}

extern "C" int __wrap_munmap(void *address, std::size_t size) {
    if (fail_release) { errno = EIO; return -1; }
    return __real_munmap(address, size);
}

int main(int argc, char **argv) {
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-scope-owned-cleanup") {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        std::array<Buffer, 2> input;
        std::array<Buffer, 2> output;
        std::array<TaskSlot, 2> slots{TaskSlot(input[0].bytes, output[0].bytes), TaskSlot(input[1].bytes, output[1].bytes)};
        Scheduler scheduler(slots, stack_bytes);
        Log log;
        check(scheduler.submit(scope_drop_failure, &log).status == ScheduleStatus::ok);
        static_cast<void>(scheduler.pump(5));
        return 3;
    }
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-owned-admission") {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        Fixture fixture;
        Buffer input;
        Log log;
        Owned capture(input.bytes);
        initialize(capture, log, 1);
        static_cast<Resource *>(capture.data())->fail = true;
        check(fixture.scheduler.submit(empty, nullptr).status == ScheduleStatus::ok);
        static_cast<void>(fixture.scheduler.submit_owned(consume, capture));
        return 3;
    }
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-owned-cleanup") {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        Fixture fixture;
        Buffer input;
        Log log;
        Owned capture(input.bytes);
        initialize(capture, log, 1);
        static_cast<Resource *>(capture.data())->fail = true;
        check(fixture.scheduler.submit_owned(produce_then_panic, capture).status == ScheduleStatus::ok);
        static_cast<void>(fixture.scheduler.pump(1));
        return 3;
    }
    if (argc != 1) { return 2; }
    for (const auto &test : cases) {
        test.run();
        std::printf("PASS %.*s\n", static_cast<int>(test.name.size()), test.name.data());
    }
    std::printf("%zu owned value cases passed\n", cases.size());
    return 0;
}
