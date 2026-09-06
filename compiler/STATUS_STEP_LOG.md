# Compiler step log

Historical work checkpoints, newest first. Read [STATUS.md](STATUS.md) for the
current handoff, validation, remaining work and ordered next steps. Older entries
record what was known at the time and may have been superseded.

## Step log

### 2026-09-06 — Finalize the bounded-list restart handoff

- State: Compiler b7ddf0c, native coverage/example abd1774 and runtime batches 4feecf8 are committed. Root/compiler STATUS now describe the completed subset, remaining language/runtime gaps and ordered continuation; historical checkpoints remain in their step logs. No source worker remains active.
- Validation: All 14 repository checks pass, including 187 Rust and 34 Python tests, runtime sanitizers and 10 passed/13 unsupported/0 failed conformance. The final optimized compiler/example passes. Final handoff documentation has 849 valid local links and clean Git whitespace.
- Blockers: no unfinished implementation or failing checks; complete v0.0.1 qualification remains outside this milestone.
- Next steps: Implement single-evaluation inference for multiple expected list alternatives; add element places, exclusive loans and move/drop state; generate task scope closing with complete failure-batch consumption. Follow the ordered STATUS next steps and preserve explicit unsupported boundaries.

### 2026-09-06 — Commit bounded lists and complete validation

- State: Compiler implementation is b7ddf0c; native coverage/example and required mixed_list conformance are abd1774; runtime failure batches are 4feecf8. All source workers have finished and the implementation is complete for the documented reference-free bounded-list subset.
- Validation: All 14 repository checks passed with 187 Rust tests, 34 Python tests and 849 local links. The final optimized compiler also built and ran the bounded-lists example in release with exact stdout and empty stderr. Conformance remains 10 passed, 13 unsupported, 0 failed; runtime sanitizers pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Refresh the final root/compiler handoff snapshots, check their links and whitespace, commit tracking separately, then continue with multi-list contextual inference, element ownership and generated task-scope cleanup in the ordered STATUS next steps.

### 2026-09-06 — Pass the integrated bounded-list gate

- State: Bounded lists and complete runtime failure batches are validated. Multi-list literal contexts remain explicit B001, obsolete unavailable-list expectations are corrected, and reference fixtures are unchanged. Runtime behavior is committed as 4feecf8; compiler code and coverage are ready for focused commits.
- Validation: All 14 combined checks passed: 110 library plus 77 native groups, 34 Python tests, 849 local links, schemas/catalog, both editors, formatting, Clippy, build and actual conformance. Runtime debug/release/ASan/UBSan/LSan suites pass. Conformance is 10 passed, 13 unsupported, 0 failed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Verify the final optimized compiler/example, commit compiler implementation separately from native examples/harness, then replace active tracking with the completed snapshot and ordered next steps.

### 2026-09-06 — Check integrated lists against the full suites

- State: Actual conformance is 10 passed, 13 unsupported, 0 failed in both profiles; mixed_list now requires its real E207 rejection. Completing-path length and typed-list union native checks pass. Final checker cleanup must replace an obsolete unsupported-list unit expectation and classify unimplemented multi-list literal inference as B001.
- Validation: Full library run: 106 passed, 1 stale-boundary expectation failed. Native run: 76 passed, 1 failed because the agreed multi-list B001 diagnostic change has not landed yet. No lowering/runtime failures; final green gate is still pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Apply those two checker boundary corrections, finish checker unit coverage/docs, run formatting/Clippy and the combined gate, then split compiler implementation from native examples/harness and final tracking.

### 2026-09-06 — Validate source-level bounded lists

- State: Eight initial native list groups pass, including accepted copies/nesting/equality/whole-list borrows, static extent/bounds rejections and dynamic P001/P003 after argument effects. Added a final completing-path proof group and typed-list union injection; multi-list expected literal inference is explicitly B001 with a typed intermediate workaround.
- Validation: Initial native list tests passed in debug/release. Backend 17 groups and all runtime profiles also passed. Final additional native cases, full frontend tests and conformance are now being checked.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Enable mixed_list in conformance REQUIRED only after actual E207 evidence, finish compiler documentation/unit tests, run the combined gate, then commit compiler behavior and coverage separately and write the final restart handoff.

### 2026-09-06 — Commit runtime batches and validate list lowering

- State: Runtime failure batches are committed as 4feecf8. Native list layout, copying, indexing, append and equality now share checked HIR target layout; checker extent arithmetic uses existing typed expression checking instead of raw i128 AST evaluation.
- Validation: Backend worker passed 17 focused backend groups, including new list/union layouts, signed and uint64 bounds, float equality and byte-span diagnostics in both profiles. Root full runtime debug/release/sanitized gate passed. Full frontend/native list tests remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish checker method wiring and frontend tests, run all native list cases and actual mixed_list conformance, document exact supported limits, then split compiler code, native coverage and final handoff commits.

### 2026-09-06 — Validate bounded runtime failure batches

- State: Detailed scope closing is complete: caller-provided batches retain every consumed failure, report_full preserves the next child and mark, and retries retain counts and ticket generation identity. Root reviewed runtime code and documentation; compiler lists remain in integration.
- Validation: Root reran python3 -B runtime/check.py outside sandbox: all debug/release/sanitized suites pass, including 25 scheduler groups, 13 owned groups and the required expired-fiber-local ASan diagnosis. The earlier isolated timeout did not recur; no sanitizer suppression was added.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit runtime batch behavior and tools documentation separately, finish compiler extent/length integration, run native/conformance and combined checks, then commit compiler behavior, examples and final tracking.

### 2026-09-06 — Review native layout and extent checking

- State: Native list lowering now snapshots receivers, checks signed and unsigned one-based indices and compares only initialized elements. Review found that draft AST-only extent arithmetic bypassed typed width checks; the checker owner is correcting that before acceptance. Bounds diagnostics now include byte spans.
- Validation: New runtime/lowering code is awaiting integrated compiler tests. Native regressions now include uint8 capacity overflow and mixed-width extent arithmetic. Link and whitespace checks passed previously; sanitizer isolation is still pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Share one checked target-layout calculation, finish width-correct required extents and known-length facts, run focused compiler and runtime checks, then run the full gate and split completed implementation/docs commits.

### 2026-09-06 — Review collection and report integration boundaries

- State: Native collection coverage now includes signed/full-width indices, typed constant capacities, initialized-prefix equality, receiver snapshots, final-use loans and explicit unsupported limits. Runtime batch implementation and borrowed diagnostic lifetime rules have been reviewed.
- Validation: 849 local documentation links and Git whitespace pass. Runtime debug/release passes; sanitizer diagnosis and compiler execution of new tests remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete compiler source integration, run focused native and conformance checks, finish sanitizer diagnosis without weakening the negative probe, then run the repository gate and split reviewed behavior commits.

### 2026-09-06 — Integrate collection evaluation with borrow analysis

- State: Borrow and loan passes now visit list receivers/items/indexes in source order, stop after nonreturning expressions and treat copied elements as reference-free. Index results get fresh union activity; whole-list reference contracts include nested element type costs without indexed borrow sources.
- Validation: Traversal edits and final-use native probes are written and formatted; compiler integration is still pending so these changes are not yet validated. Runtime sanitizer probe diagnosis remains pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish frontend/backend integration, run cargo tests and inspect any inference or lowering failures, verify retry batches under sanitizers, then enable actual mixed-list rejection and split completed commits.

### 2026-09-06 — Add bounded-list native scenarios

- State: Eight native groups and the bounded-lists example now specify copy behavior, typed inference, nested values, initialized-prefix equality, snapshots, bounds and whole-list loans. Frontend and backend work are split between workers; root owns integration tests and docs.
- Validation: New native tests are written but cannot run until compiler integration is complete. Runtime debug/release passes 25 scheduler groups; sanitizer run stopped at a timeout in the existing expired-fiber-local negative probe and is being isolated.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish and compile list passes, execute new native scenarios in both profiles, resolve the sanitizer probe timeout with exact evidence, then run the combined gate and split behavior commits.

### 2026-09-06 — Implement retry-safe failure batches

- State: Runtime detailed scope closing now keeps a failed child pending when the caller buffer is full and reports only successfully reclaimed failures. Compiler bounded-list lowering is still in progress; root is adding native behavior coverage.
- Validation: Runtime debug/release passes with 25 scheduler groups and existing suites. Compiler baseline remains 104 library plus 68 native groups; new lists and sanitizer results are pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete runtime sanitizer checks and buffer lifetime documentation, finish bounded-list HIR/checker/lowering, then run native bounds, inference, equality and evaluation-order tests before enabling mixed_list conformance.

### 2026-09-06 — Begin bounded lists and complete scope-failure reporting

- State: The checkout is clean at 4a01cf6. Compiler work targets inline bounded lists with copyable reference-free elements, initialized length, checked one-based indexing and compatible literal inference. Runtime work targets bounded batches of every consumed child failure during explicit scope close. Root owns native/conformance coverage and both trackers.
- Validation: Previous milestone passed all 14 repository checks. New collection/report behavior is not yet implemented or validated. Slices, owned/reference elements, element mutation/borrowing, core.Type evaluation, automatic cancellation and DWARF remain separate.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Define bounded list HIR/layout and static versus runtime bounds checks, design retry-safe failure report batches, add native accepted/rejected examples and enable conformance cases only after actual compiler support.

### 2026-09-06 — Complete scoped-borrow and task-close handoff

- Completed: compiler 393471c, native coverage efe7d6e and runtime/tooling d7d1758 are committed. Current STATUS files contain supported behavior, precise limits, evidence and ordered next steps; prior checkpoints are preserved.
- Validation: All 14 repository checks pass: 172 Rust tests, 34 Python regressions, 848 links and every native/sanitizer profile. The optimized example has exact stdout; all 108 release matrix cases pass (66 accepted, 42 E302). Final tracker links and whitespace pass.
- Blockers: none for this milestone. Conformance remains 9 passed, 14 unsupported, 0 failed. Exclusive/new-source capabilities, generated cleanup, automatic joins/cancellation and DWARF remain pending.
- Next steps: extend source contracts before new reference capabilities; add read/move/initialization/cleanup edges; generate runtime mark/close calls and handle failure reports while locals live; continue cancellation/unwinding and module/library implementation. Update STATUS and add a checkpoint after each logical step.

### 2026-09-06 — Refresh scoped-borrow and task-close handoff

- State: Current snapshots now identify compiler 393471c, native coverage efe7d6e and runtime/tooling d7d1758. Completed parameter/dispatch/carrier and explicit-close work is separated from remaining generated cleanup, cancellation and new-source capabilities. Prior checkpoints remain intact.
- Validation: All 14 checks, optimized example and 108 release matrix cases pass. Final tracker link/whitespace checks are next.
- Blockers: none for this milestone; full-language and automatic cleanup/unwind integration remain pending.
- Next steps: Verify and commit the handoff, then resume the ordered source-contract, ownership, generated scope-exit and module work in STATUS.

### 2026-09-06 — Commit explicit task scope closing

- State: Runtime opaque scope marks, fixed metadata, waiting child cleanup, owned-result discard and cumulative retry-safe failure reports are committed with tests and tooling. Compiler and coverage commits are 393471c and efe7d6e. Only current STATUS snapshots and step logs remain uncommitted.
- Validation: All runtime debug/release/sanitizer profiles and the combined repository gate pass; staged whitespace checks pass. No active source edits or known defects remain.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Refresh root/compiler STATUS with the final commit IDs, exact evidence and remaining generated cleanup/cancellation work; check links/whitespace and commit the handoff.

### 2026-09-06 — Commit scoped-borrow native coverage

- State: Compiler implementation is 393471c and native/example/usage coverage is efe7d6e. They preserve parameter/self local-copy lifetimes, original shared referents, inherited bounds, nullable carriers and once-only reference-prefix evaluation. Runtime scope close and the final handoff remain.
- Validation: All 104 library and 68 native groups, optimized example and 108 release matrix cases pass. Staged whitespace checks pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit explicit task scope closing with runtime tests/tooling, then refresh current STATUS snapshots and validate the final documentation handoff.

### 2026-09-06 — Commit scoped parameter and receiver borrowing

- State: Compiler parameter/self storage, reference-bearing dispatch, carrier-prefix reborrows, source regressions and ownership rules are committed. The existing native capability case now covers forbidden holder address-taking. New native/example coverage, runtime scope close and handoff remain.
- Validation: The complete gate, optimized example, 108-case release matrix and staged whitespace checks passed. No source changes are pending outside the remaining reviewed slices.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit scoped-borrow native coverage and example, then runtime close and the final handoff with exact implementation commit IDs and remaining cleanup/cancellation work.

### 2026-09-06 — Pass the final optimized dispatch matrix

- State: The final optimized compiler passes all 108 implicit dispatch/carrier guard cases on a private snapshot. Implementation, native behavior and runtime review are complete with no remaining blocker. Root is preparing dependency-ordered compiler, coverage, runtime and handoff commits.
- Validation: Release matrix: 66 accepted, 42 expected E302, no conservative or unexpected results. The optimized example, all 14 repository checks, 172 Rust tests, 34 Python tests and 848 links pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit scoped parameter/receiver borrowing and existing boundary correction, then native/example docs, explicit runtime scope close and final current STATUS snapshots/step logs.

### 2026-09-06 — Verify the optimized scoped-borrow example

- State: The optimized compiler builds and runs scope-borrows.mwy with exact output. Compiler and runtime source review is complete, and all integration checks are green. Root is preparing focused implementation/coverage/runtime commits before the final tracker snapshot.
- Validation: All 14 checks pass: 172 Rust tests, 34 Python tests, 848 links and runtime profiles. Release example stdout is false, 7, 8, true, 9, 10 on separate lines with empty stderr. Final optimized dispatch matrix will confirm the release snapshot.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the final 108-case release matrix, commit compiler and existing boundary correction, native/example docs, runtime scope close and final handoff in separate slices.

### 2026-09-06 — Pass the complete scoped-borrow and task-close gate

- State: All 14 repository checks pass with parameter/self borrowing, reference-bearing dispatch, carrier-prefix reborrows and explicit task scope closing integrated. Root is building the optimized compiler before final commits; no pending source edits or failing checks remain.
- Validation: 104 library plus 68 native Rust groups, 34 Python regressions, 848 links, schemas/catalog, editors, formatting/Clippy/build and conformance pass. Runtime all profiles pass 22 scheduler and 13 owned groups plus exact fatal probes. Conformance remains 9 passed, 14 unsupported, 0 failed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the optimized scope-borrows example and final dispatch guard matrix, then commit compiler implementation, native coverage, runtime scope close and refreshed STATUS handoff separately.

### 2026-09-06 — Pass dispatch guard matrix and complete scope-close validation

- State: The fresh compiler snapshot passes the implicit carrier-chain dispatch matrix and all directed lifetime checks. Runtime scope closing is fully validated with fixed marks, child waits, owned-result release and retry-safe failure reporting. No implementation blocker remains.
- Validation: Dispatch matrix: 66 accepts, 42 expected E302 across 108 cases, no conservative or unexpected results. Nineteen directed checks pass. Runtime all profiles pass 22 scheduler and 13 owned groups plus exact fatal probes; 14 runtime Python tests pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run all 14 repository checks, build and exercise the optimized scope-borrows example, rerun final snapshot probes as needed, then commit compiler, native coverage, runtime scope-close and handoff slices.

### 2026-09-06 — Qualify runtime scope close and finish compiler carrier contracts

- State: Runtime scope-close work is complete and validated, including nested marks, owned child-result discard, cumulative failure reports and retry-safe reclamation. Root added a direct-function carrier-prefix return regression and documented implicit reference-prefix reborrows. Tools now describe explicit scope-close coverage.
- Validation: All runtime debug/release/sanitizer profiles pass 22 scheduler and 13 owned groups plus exact fatal probes; 14 runtime Python tests and 848 links pass. The new compiler source regression and tooling regressions are running; prior 104 library and 68 native groups are green.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete independent compiler guard/snapshot review, run the full repository gate, then verify the optimized example and commit compiler, coverage, runtime and handoff slices.

### 2026-09-06 — Pass carrier-prefix reborrows and review retry-safe scope close

- State: All 68 native groups pass after adding reference-prefix traversal. Root reviewed fixed runtime scope records, LIFO marks, child boundaries, post-reclamation failure counts and owned-result discard; no lifecycle defect was found. Scope close keeps failed children and reports progress without double counting.
- Validation: Compiler native debug/release passes. Runtime debug/release passes 22 scheduler and 13 owned groups plus new unclosed-scope/fatal discard probes; 14 runtime Python tests pass. Sanitizer and final independent compiler checks remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete scoped dispatch guard-matrix and sanitizer validation, document carrier-prefix and close-report limits, then run the full repository gate and optimized example before focused commits.

### 2026-09-06 — Support reborrowing through reference-valued carrier prefixes

- State: Root extended address lowering for chains such as &holder.view.field: leading carrier fields produce a shared reference value once, then existing reborrow logic addresses its referent. Addresses of holder reference slots or scalar fields remain B001. New native coverage includes effectful temporary carriers, nullable references and E302/E303 boundaries.
- Validation: All 104 library and 67 native groups passed before this addition. Formatting passes; the new carrier-chain group is running. Independent matrix review will retain the original implicit chain syntax.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Pass carrier-prefix identity/evaluation tests, rerun the full native suite and independent guard matrix, then finish runtime scope-close validation and the combined gate.

### 2026-09-06 — Pass library checks and document parameter/self storage regions

- State: All 104 library groups pass. Ownership documentation now distinguishes copied parameter/self storage from shared receiver referents, preserves E303 local-address rejection and describes reference-carrier dispatch without inventing a function contract. Runtime scope records and retry-safe close logic are implemented.
- Validation: All 67 native groups and 104 library groups pass; all-target Clippy is running. Independent snapshot probes and runtime close-specific lifecycle/sanitizer checks remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish independent compiler verification and runtime scope-close tests, update tool/runtime scope descriptions, then run the full repository gate and optimized example before focused commits.

### 2026-09-06 — Add source regressions and scope-borrow usage documentation

- State: Root added source regressions for parameter/receiver copy escapes and shared-dispatch provenance, and documented the scope-borrows example. The compiler reuses local storage and existing origin/bound analysis without new lifetime shortcuts.
- Validation: All 67 native groups passed before this source-test addition. Formatting completed; updated library tests are running. Independent snapshot review and runtime scope-close implementation continue.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish library and independent guard/lifetime probes, document exact storage boundaries, then validate runtime scope-close cleanup/retry semantics and run the combined repository gate.

### 2026-09-06 — Pass scoped parameter and dispatch native behavior

- State: All 67 native groups pass. Parameter/self addresses retain local-copy identity and reject escapes; shared/reference-carrier dispatch preserves original owners, nullable activity and inherited bounds. Receiver/argument evaluation remains once-only and left-to-right. Independent diff review found no source blocker.
- Validation: Native debug/release passes, including the new scope-borrows example and E302/E303 boundaries. A fresh standalone compiler is ready for focused review probes. Runtime scope-close implementation is still in progress.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run independent snapshot probes, add focused source regressions and update borrowing docs; then validate scope nesting, close retries, failure reports and owned child-result cleanup before the combined gate.

### 2026-09-06 — Specify parameter and dispatch lifetime behavior

- State: Root added scope-borrows.mwy and six native groups covering parameter-copy identity, left-to-right arguments, borrowed record fields, copied versus shared self, reference-carrier/nullable dispatch, once-only evaluation, inherited bounds and E303 escapes. Shared dispatch conflicts remain E302.
- Validation: New native coverage has 67 groups pending execution; baseline 102 library/61 native groups passed. Runtime scope-close design preserves counts across retries and reports unreclaimed children separately, with fixed scope metadata.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run new native/source groups, verify copied parameter/self storage never becomes caller-owned provenance, then validate runtime scope nesting, result destruction, failure accumulation and retry behavior.

### 2026-09-06 — Enable scoped parameter and receiver addresses

- State: The checker now admits reference-free parameter/self storage as local places and allows shared-reference/reference-bearing-value dispatch blocks to use existing provenance analysis. The first library run found only obsolete B001 expectations; those now cover exclusive parameter borrowing and reference-carrier address-taking. Runtime scope marks use fixed records and retry-safe close reports.
- Validation: Baseline 102 library and 61 native groups passed. The new code compiles; its first library run passed 101 groups with one stale capability-test group failing. New accepted/E303/E302 coverage is next.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Verify local-copy identity and escape rejection, shared self provenance and inherited bounds, then implement bounded nested scope close with owned-result cleanup, failure reporting and exact fatal probes.

### 2026-09-06 — Begin parameter and dispatch borrowing with explicit task scope close

- State: The checkout is clean at b21496d. Root is extending addressable storage to by-value parameters and dispatch self bindings, and enabling reference-bearing dispatch through existing origin analysis. Runtime work targets explicit scope closing while parent locals still live, with child/result cleanup and retry-safe ownership. Root is the sole tracker writer.
- Validation: Previous milestone passed all 14 checks. New parameter/dispatch and scope-close behavior is not implemented or validated yet. Exclusive access, automatic destructor/scope-exit joins, cancellation and DWARF remain separate.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Confirm copied receiver/parameter lifetimes against the reference, implement scoped addresses and dispatch provenance, define bounded scope marks/close semantics, then add accepted/E303/E302 and child cleanup/retry tests.

### 2026-09-06 — Complete reborrow and owned-runtime handoff

- Completed: shared reborrows are 406c817, native example/coverage is 21086be, and owned runtime/tooling is 979c8e8. Current STATUS files record exact behavior, capability limits, verification and ordered next steps; all prior checkpoints are preserved.
- Validation: All 14 repository checks pass: 163 Rust tests, 33 Python regressions, 847 links and every native runtime/sanitizer profile. The optimized example has exact stdout; all 150 release oracle cases pass (114 accepted, 36 E302). Final tracker links and whitespace pass.
- Blockers: none for this milestone. Worker usage limits were handled by root without leaving incomplete code. Conformance remains 9 passed, 14 unsupported, 0 failed; exclusive/new reference sources, generated cleanup, automatic joins/cancellation and DWARF remain pending.
- Next steps: extend source contracts before enabling new addressable/static/intrinsic references; add read/move/initialization/cleanup edges; generate owned payload layouts and scope-exit joins while locals live; continue cancellation/unwinding and module/library implementation. Update STATUS and add a checkpoint after each logical step.

### 2026-09-06 — Refresh shared-reborrow and owned-runtime handoff

- State: Current snapshots now identify compiler 406c817, native coverage 21086be and runtime/tooling 979c8e8. Completed shared-reborrow and owned-payload priorities were replaced by exclusive/new-source work, generated cleanup, automatic joins and cancellation. All prior checkpoints remain intact.
- Validation: All 14 combined checks, optimized example and 150 release oracle cases pass. Final tracker links and whitespace are next.
- Blockers: none for this milestone; full-language and automatic cleanup/unwind integration remain pending.
- Next steps: Verify and commit the handoff, then resume the ordered source-contract, ownership, runtime and module work in STATUS.

### 2026-09-06 — Commit owned runtime capture and result transfer

- State: Owned runtime storage, scheduler integration, lifecycle/fatal tests, rules and tooling are committed. Compiler implementation and native coverage are 406c817 and 21086be. Only current root/compiler handoff files and their step logs remain uncommitted.
- Validation: All runtime profiles pass 12 owned-value groups, two owned-cleanup fatal probes and prior suites. The complete 14-check gate and staged whitespace checks pass. No source edits or active implementation workers remain.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Refresh current STATUS with the final three implementation commits, exact validation and remaining exclusive/static/cleanup/cancellation work; check links/whitespace and commit the handoff.

### 2026-09-06 — Commit reborrow native coverage and example

- State: Compiler implementation is 406c817 and native/example/usage coverage is 21086be. Coverage protects original field addresses, single evaluation, control-flow exits, inherited bounds, guarded parents and E302/E303 rejection. Runtime ownership and final handoff are next.
- Validation: All 61 native groups, 102 library groups, optimized example and 150-case release oracle pass. Staged whitespace checks pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit owned capture/result storage with scheduler tests and tooling, then refresh and validate both current STATUS snapshots and step logs before the handoff commit.

### 2026-09-06 — Commit shared reborrow implementation

- State: The compiler shared-reborrow implementation is committed, including dedicated sites, pure address hints, direct field addresses, referent projection sources, inherited bounds and bounded function-source enumeration. Existing exclusive/temporary/union-payload boundaries remain explicit.
- Validation: All combined and optimized checks passed before commit; staged whitespace checks pass. New native coverage/example, runtime ownership and current handoff remain to commit.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit reborrow native coverage/example and usage docs, then owned runtime/tooling; finish current handoff and next steps with exact commit IDs.

### 2026-09-06 — Pass optimized reborrow execution and prepare final commits

- State: The optimized compiler executes reborrows.mwy with exact output and passes all 150 guard-oracle cases. Runtime owned storage and source review are complete with no remaining blocker. Root is preparing coherent compiler, coverage, runtime and handoff commits.
- Validation: Release oracle: 114 accepted, 36 expected E302, zero failures. Exact example stdout: true, 41, 7, 42, 8 on separate lines. All 14 combined checks, 163 Rust tests, 33 Python tests, runtime profiles and 847 links pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit shared reborrow implementation, then native/example documentation, then owned-runtime/tooling; refresh current STATUS snapshots with those commits and remaining cleanup/cancellation work, validate and commit the handoff.

### 2026-09-06 — Pass the complete reborrow and owned-runtime gate

- State: All 14 repository checks pass with shared reborrows, projected function sources and owned runtime capture/result transfer integrated. No pending source edits or failing checks remain. Root is building the optimized compiler and preparing focused commits.
- Validation: 102 library plus 61 native Rust groups, 33 Python regressions, 847 links, schemas/catalog, editors, formatting/Clippy/build and conformance pass. Runtime debug/release/ASan/UBSan/LSan include 12 owned-value groups and two owned-cleanup fatal probes plus all prior suites. Conformance is 9 passed, 14 unsupported, 0 failed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run reborrows.mwy with the optimized compiler and exact output, rerun the 150-case guard oracle on that binary, then commit compiler, native coverage, runtime ownership and refreshed STATUS handoff separately.

### 2026-09-06 — Complete owned runtime validation and start final integration

- State: Owned capture/result storage now passes all runtime profiles with real relocation, consumed admission failures, panic cleanup order, child-safe movement and result-preserving join retries. Compiler shared reborrows and projected function sources are complete. Root is starting the combined gate with no pending source edits.
- Validation: Compiler: 102 library groups, 60 earlier native groups plus the new control-flow group, 150 oracle cases, formatting and Clippy pass. Runtime: 12 owned groups and two exact fatal probes plus all prior suites pass in debug/release/ASan/UBSan/LSan. Thirteen runtime and 16 tooling Python tests and 847 links pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run all 14 repository checks, verify the optimized reborrow example and final 150-case oracle, then commit compiler implementation, native coverage, runtime ownership and current handoff as focused slices.

### 2026-09-06 — Pass compiler projection regressions and document reborrow limits

- State: All 102 library groups and the new native early-leave/effect test pass. Shared reborrow docs now explain original addresses, separate referent projections, inherited bounds, call-source enumeration and explicit unsupported pointee/union/static cases. Root corrected two test-string escaping errors before these passes.
- Validation: Clippy with warnings denied and formatting pass; the 150-case oracle and 60 earlier native groups pass. Runtime owned values pass debug/release with 12 new groups, including OS-resource relocation and exactly-once cleanup; sanitizer and final fatal probes remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish owned runtime sanitizer/docs checks, review all final diffs, then run the combined gate and optimized reborrow example/oracle before focused implementation, coverage, runtime and handoff commits.

### 2026-09-06 — Pass the reborrow guard oracle and extend source regressions

- State: The 150-case oracle passes for direct and function-returned field reborrows. Root added public-source regressions for nested input projections, inherited ignored-input lifetime bounds, field-candidate expansion limits, and native early-leave behavior in an effectful reborrow equality operand.
- Validation: Oracle: 114 accepted, 36 expected E302, zero failures. All 60 previously added/native baseline groups passed. Formatting found two incorrectly escaped new Rust test strings; root is correcting them before running updated library and control-flow checks. Runtime owned transfer tests are under construction.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Pass projected-source/budget and control-flow regressions, review runtime owned relocation/drop behavior, update ownership/usage docs, then run final combined verification and optimized examples/oracles before commits.

### 2026-09-06 — Pass native reborrow identity and lifetime checks

- State: All 60 native groups pass, including original nested addresses, field-returning function contracts, single evaluation, inherited call bounds, guarded parents and E302/E303 boundaries. Root prepared a 150-case truth-table oracle for direct and function-returned field references.
- Validation: Native debug/release passes. The only earlier library failure was the corrected obsolete B001 expectation for shared &*ref. Runtime owned scheduling APIs are implemented but not yet validated.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the reborrow oracle on a fresh binary, add focused call-field/budget regressions and formatting/Clippy, then review owned runtime transfer semantics and execute the combined gate.

### 2026-09-06 — Preserve exclusive-reborrow rejection after enabling shared reborrows

- State: The first compiler run passed 99 of 100 existing library groups; the sole failure was the old expectation that &*ref is unsupported. That boundary case now checks exclusive &!*ref, which remains B001. Root is running the full 60-group native suite.
- Validation: No behavioral library regression appeared beyond the obsolete shared-reborrow capability expectation. Native address identity, field-return contracts and lifetime behavior are being exercised next; runtime owned storage is still under implementation.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Resolve native failures, add focused projected-source and budget regressions, run a guarded reborrow oracle and formatting/Clippy, then complete runtime ownership validation and integrated checks.

### 2026-09-06 — Implement shared reborrow lowering and projected call sources

- State: Root implemented explicit HIR reborrow sites, pure address hints, one-time parent evaluation and direct field-address lowering. Actual Local/Input sources now carry referent projections separately; inherited lifetime bounds remain unchanged. Call contracts enumerate compatible concrete named descendants before allowing field-returning functions.
- Validation: cargo check passes. Source and native regressions are starting; runtime owned storage remains with its active worker. No static references, exclusive loans, union payload addresses or reference-bearing pointees were enabled.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the 60 native groups and library tests, correct obsolete capability expectations, audit projected-source/liveness boundaries and resource caps, then integrate owned runtime storage and final verification.

### 2026-09-06 — Take over compiler work after worker usage limits

- State: The compiler and review workers stopped on usage limits after design, before editing compiler sources. Root is taking over shared reborrow implementation. Native coverage and example remain intact; runtime ownership design is approved and its worker was notified to report availability before handoff.
- Validation: Baseline 100 library and 54 native groups passed. No partial compiler source migration exists yet. New 60-group native suite remains pending shared reborrow support; no validated work was lost.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement dedicated HIR reborrow/address projection, symbolic referent fields and bounded call-source enumeration, run targeted/native checks, then complete runtime ownership work and final repository validation.

### 2026-09-06 — Preserve pure type hints and single evaluation for reborrow operands

- State: Independent review identified that generalized address resolution must not execute through the pure type-hint path. Root added effectful reborrow calls inside equality, complementing direct field/call cases, so lowered operands must run exactly once.
- Validation: Baseline compiler checks pass. New reborrow native coverage now has 60 groups and remains unvalidated pending implementation. Existing native source fixtures were preserved; no new borrow syntax or implicit allocations were introduced.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Keep address hints pure while lowering reborrow operands once, preserve projected origins and inherited bounds, finalize bounded owned-payload admission semantics, then run focused native and ownership checks.

### 2026-09-06 — Specify shared reborrow execution and lifetime coverage

- State: Root added reborrows.mwy and six native groups covering identity, nested field addresses, temporary reference-valued calls, once-only evaluation, guarded/nullable parents, loops, inherited call bounds and E302/E303 rejection. Temporary owners, exclusive reborrows and reference-carrying storage remain explicit capability boundaries.
- Validation: Baseline 100 library and 54 native groups pass. New reborrow cases are specifications awaiting implementation. Runtime ownership design must release accepted transferred captures on failed admission and reserve result storage before join.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete projected source/address representation and call-compatible field enumeration, settle runtime owned storage/move/failure contracts, then execute the new native cases and ownership lifecycle probes.

### 2026-09-06 — Begin shared reborrows and owned task payload work

- State: The checkout is clean at 0294d5b. Compiler work targets shared reborrows and concrete field references through shared record inputs, with projected call provenance before enabling those returns. Runtime work is designing a bounded explicit capture/result ownership API. Root owns native examples/coverage and both trackers; independent review audits provenance and lifetimes.
- Validation: The previous milestone passed all 14 repository checks. New reborrow/owned-payload behavior is not yet implemented or validated. Exclusive access, reference-carrying referents, static/intrinsic sources and automatic cleanup/unwinding remain separate unless fully proven.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Define referent projection paths distinct from carrier component paths, implement shared address-preserving reborrows and call-source substitution, settle runtime ownership/failure contracts, then add E302/E303 and exactly-once cleanup coverage.

### 2026-09-06 — Complete function-contract and child-wait handoff

- Completed: compiler contracts are 3acfc97, native example/coverage is cb04d32, and child-wait runtime/tooling is 7b9dc05. Current STATUS files contain supported behavior, capability boundaries, exact evidence and ordered next steps; prior checkpoints are preserved.
- Validation: All 14 combined checks pass: 154 Rust tests, 33 Python regressions, 845 links and every runtime debug/release/sanitizer profile. The optimized example has exact stdout, and all 384 release oracle cases pass (216 accepted, 168 E302). Final tracker links and whitespace pass.
- Blockers: none for this milestone. Conformance remains 9 passed, 14 unsupported, 0 failed. Reborrows, exclusive ownership, generated cleanup, owned capture/result storage, automatic joins/cancellation and DWARF remain pending.
- Next steps: extend reference-source contracts before enabling reborrows/static/intrinsic origins; add read/move/initialization/cleanup edges; generate scope-exit joins while locals live; implement cancellation/unwinding and continue module/library work. Keep STATUS current with a checkpoint after each logical step.

### 2026-09-06 — Refresh function-contract and child-wait handoff

- State: Current snapshots now identify compiler 3acfc97, native coverage cb04d32 and runtime/tooling 7b9dc05. Completed function/child-wait priorities were replaced by reborrows, new reference sources, ownership cleanup and automatic scope-exit joins/cancellation. Prior checkpoints remain intact.
- Validation: All 14 combined checks, optimized borrow-functions output and all 384 release oracle cases pass. Final tracker link/whitespace verification is next.
- Blockers: none for the completed milestone; full-language, automatic cleanup and unwind integration remain pending.
- Next steps: Verify the final tracker links/whitespace, commit the handoff, then resume the ordered source-contract, ownership and runtime work in STATUS.

### 2026-09-06 — Commit function-borrow native coverage

- State: Native function-borrow coverage, borrow-functions.mwy and compiler usage docs are committed after compiler 3acfc97 and runtime 7b9dc05. They cover inference/recursion, temporary carriers, nullable results, transitive bounds, scalar-only release and E302/E303 boundaries. Only handoff files remain.
- Validation: All 54 native groups pass in both profiles. The optimized example and 384-case release contract oracle pass, as do staged whitespace checks.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Refresh current root/compiler STATUS with the three implementation commits, completed evidence and ordered reborrow/cleanup/automatic-join work; validate links and whitespace, then commit the handoff.

### 2026-09-06 — Commit direct-function contract implementation

- State: Compiler direct-function contracts, symbolic parameter sources, separate lifetime bounds, call-site proofs, resource limits and ownership design are committed as 3acfc97. The existing capability tests now cover carrier address-taking and mutable reference storage. Runtime child waits are 7b9dc05.
- Validation: All 14 combined checks, optimized example, 384 release oracle cases and staged whitespace checks pass. Only new native/example/usage coverage and current tracker handoff remain uncommitted.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit the function-borrow native coverage and example, then update both STATUS snapshots with final commits, exact evidence and next ownership/runtime work before the documentation commit.

### 2026-09-06 — Pass the final release contract oracle and prepare coherent commits

- State: The optimized function oracle passes all 384 cases on a private release snapshot. Source, native behavior and runtime review are complete with no remaining blocker. Root prepared the compiler slice with only the two existing unsupported-boundary corrections and their formatter-compatible layout; new native coverage stays separate.
- Validation: Release oracle: 216 accepted, 168 expected E302, zero conservative or unexpected results. Optimized example output, all 14 combined checks and whitespace checks pass. Runtime is already committed as 7b9dc05.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit compiler contracts and their existing boundary updates, then native coverage/example, then refresh and commit both current STATUS snapshots and step logs with precise remaining work.

### 2026-09-06 — Verify optimized function borrow execution

- State: The optimized compiler builds and runs borrow-functions.mwy with exact output. Final source review and the combined gate are green. Runtime child waits are committed as 7b9dc05; root is preparing compiler and native coverage slices while the reviewer audits the final release snapshot.
- Validation: All 14 checks pass: 154 Rust tests, 33 Python tests and 845 links plus runtime profiles. Release example stdout is 11, 22, 12, 23, 12, 24 on separate lines with empty stderr. The final 384-case release oracle is running.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Confirm the optimized oracle result, commit compiler implementation with existing capability-boundary corrections, then native example/coverage and refreshed root/compiler STATUS snapshots and logs.

### 2026-09-06 — Pass the combined function-contract and child-wait gate

- State: All 14 repository checks pass with direct-function borrow contracts and explicit child waits integrated. No source edits or failing checks remain. Root is building the optimized compiler before final commit slices; automatic cleanup/cancellation and full language support remain pending.
- Validation: 100 library plus 54 native Rust groups, 33 Python regressions, 845 links, schemas/catalog, editors, formatting/Clippy/build and conformance pass. Runtime debug/release/ASan/UBSan/LSan pass 18 scheduler groups and all exact fatal/guard/admission probes. Conformance remains 9 passed, 14 unsupported, 0 failed in both profiles.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Verify borrow-functions with the optimized compiler and exact stdout, rerun the 384-case contract oracle on its completed binary, then commit compiler implementation, native example/coverage and refreshed STATUS handoffs separately.

### 2026-09-06 — Complete function contract implementation and start the full gate

- State: Compiler function contracts are complete: symbolic Local/Input sources, separate conservative bounds, call snapshots, scoped normal-return proofs and bounded source/type expansion. Runtime explicit child joins are committed as 7b9dc05. All workers have finished source edits and root is running final integration.
- Validation: All 100 library and 54 native groups pass; formatting, all-target Clippy and whitespace checks pass. The 384-case oracle, 12 directed semantic checks and 10 extra native runs are green. Runtime profiles and Python checks are green independently.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run all 14 repository checks, then build the optimized compiler and borrow-functions example, rerun the function oracle on its stable binary, and commit compiler implementation, native coverage and the current handoff separately.

### 2026-09-06 — Pass the independent function contract oracle and directed probes

- State: The independent function oracle passes on a private compiler snapshot, and all directed lifetime/divergence/argument-control probes match the reference. Function calls retain active all-input bounds without retaining absent inputs or scalar-only results. Loan source integration is complete with no outstanding review issue.
- Validation: Oracle: 216 accepts, 168 expected E302, zero conservative or unexpected results across 384 cases. Twelve directed checks pass, including E303 ignored-input escape, unproductive recursion, branch-isolated divergence, invalid uncalled bodies and safe leave/panic arguments. All 19 loan groups pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish the compiler owner documentation/resource handoff, execute accepted directed programs in both profiles, then run final formatting/Clippy and the complete repository gate before optimized example/oracle and focused compiler commits.

### 2026-09-06 — Pass all native function contract groups

- State: All 54 native groups pass with direct-function borrow contracts: identity, inference, aliases, recursion/forward groups, record/union carriers, nullable output, scalar-only release, transitive all-input loans and local/ignored-input escapes. A fresh standalone compiler build is ready for independent audit.
- Validation: Native execution/rejection checks pass in debug/release, including the new borrow-functions example. Source/library, resource and independent oracle review are continuing; runtime 7b9dc05 remains qualified.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the 384-case oracle on a private binary snapshot and directed lifetime/divergence/argument-order probes, finish source/resource tests and docs, then run formatting/Clippy and all 14 repository checks.

### 2026-09-06 — Compile direct-function borrow contracts and call liveness

- State: Function source and CFG interfaces now compile. Symbolic parameter origins, separate all-input/transitive bounds, call-site snapshots and entered-guard continuation proofs are integrated. Type-comparison fanout is bounded during expansion. Root is running all 54 native groups while workers run focused source tests.
- Validation: Compilation passes; native and source function behavior are now being validated. Runtime child waits remain independently committed and qualified as 7b9dc05. The 384-case oracle is prepared with expected 216 accepts and 168 E302 results.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Resolve any native/source regressions, build a stable compiler snapshot for the oracle and directed E303/divergence/argument-order audit, then complete docs and full repository verification.

### 2026-09-06 — Exercise inferred and mutually recursive borrow signatures

- State: Root extended native coverage with inferred reference result types and strict forward groups that return shared references through mutual recursion. Runtime child waits are committed as 7b9dc05. Compiler value/source interfaces and CFG calls are being adapted in parallel.
- Validation: Baseline tests pass; new function groups have not yet run because borrow analysis is incomplete. Tests follow the existing forward signature syntax and named early-exit rules, without changing reference fixtures.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish signature-based call substitution and symbolic parameter entry, then run native identity/recursion/carrier tests and all-input E302/E303 cases; execute the independent oracle before the combined gate.

### 2026-09-06 — Document function contract scope while call analysis is integrated

- State: Runtime child waits are committed as 7b9dc05. Compiler README now describes the intended direct-function borrow contract, the new example and all-input retention versus scalar-only release. Actual source origins remain distinct from conservative lifetime dependencies; full call analysis is still being completed.
- Validation: Baseline compiler checks and committed runtime profiles pass. New function execution is unverified; no implicit borrowing, exclusive access, mutable carriers, captures or indirect calls are being enabled by the documentation.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish the origin/bound and call-site integration, then run the 54 native groups, independent call oracle and focused local-escape/recursive/argument-order checks before final verification and commits.

### 2026-09-06 — Commit explicit parent-owned child waits

- State: The independently validated runtime/tooling slice is committed: active parents can spawn and wait for direct children, waiting yields the worker, release retries preserve ownership, and missing joins fail before expired parent storage can be reused. Compiler origin/bound and CFG call integration continue in separate files.
- Validation: All runtime debug/release/sanitizer profiles pass, including 18 scheduler groups and exact missing-join probes. Thirteen runtime and 16 tooling Python regressions pass; staged whitespace checks pass. Compiler contract execution is still pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish compiler function contracts and call liveness, run all 54 native groups and the 384-case oracle, then perform the combined gate and optimized example before compiler/coverage/handoff commits.

### 2026-09-06 — Qualify explicit child joins and hand off call liveness

- State: Runtime explicit child admission and waiting joins are complete: exact parent ownership, waiting-ticket wakeups, retained release failures and missing-join protocol checks are documented. The runtime worker now owns compiler/loans.rs call-contract integration while the compiler owner adapts source origins/bounds.
- Validation: All runtime profiles pass 18 scheduler groups plus exact kernel/fatal/unjoined-child probes and prior cleanup/stack/context checks. Thirteen runtime and 16 tooling Python tests pass; 844 local links and runtime whitespace pass. Compiler contract code has temporary interface errors and is not yet build-qualified.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit the independently validated runtime/tooling slice; finish symbolic input/call snapshots and CFG bounds, then run all 54 native groups and the planned 384-case call-contract oracle plus directed E303/recursion tests.

### 2026-09-06 — Pass child-wait debug and release profiles

- State: Runtime child admission/waiting joins pass debug/release, including nested parent-local borrows, cleanup-local joins, owner checks and retained release failures. Root added transitive function-bound cases so forwarding a returned view or selecting one field of an input carrier cannot drop ignored input dependencies.
- Validation: All 18 scheduler groups, exact unjoined-child protocol probes and prior runtime suites pass in debug/release; 13 runtime Python regressions pass. One initializer compile failure was corrected before these passes. Sanitizers and compiler contract execution remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish runtime sanitizer/docs checks; land Local/Input sources, State bounds and call IDs, then delegate CFG call integration and run native function/escape/liveness cases on a fresh compiler.

### 2026-09-06 — Cover nullable call results and independent call bounds

- State: Root added nullable/reference-union result coverage, zero-input null returns, scalar extraction before a short-lived ignored input ends, and independent calls whose projected fields keep separate dependencies. Native coverage now has 54 groups pending the function implementation. Runtime has 18 scheduler groups plus private unjoined-child fatal probes ready for validation.
- Validation: Baseline 90 library and 46 native groups pass. New function cases and child-wait profiles are not yet qualified. The reviewer confirms separate typed origins and all-input bounds are required; absent inputs contribute no loan.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete call snapshots and parameter origin validation, preserve nullable output activity and scalar-only releases, then run native function and structured-wait checks with exact fatal/ASan evidence.

### 2026-09-06 — Specify function contract execution and child waiting boundaries

- State: Root added borrow-functions.mwy and six native groups for pointer identity, aliases, scalar-return final use, temporary reference carriers, recursion, all-input E302 retention and E303 escapes through ignored inputs. Compiler design separates actual Local/Input sources from conservative State bounds. Runtime child waiting states are implemented and tests are being added.
- Validation: Baseline 90 library and 46 native groups pass. New compiler cases await implementation. Runtime will require explicit child joins before C++ body/cleanup locals expire; returning with outstanding children will fail with a private protocol diagnostic rather than misuse a language panic code.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement symbolic parameters, signature-based caller substitution and call-site snapshots, then integrate CFG result bounds; verify waiting parents yield workers, only owners join children, and release failures retain child ownership.

### 2026-09-06 — Begin function borrow contracts and child joins

- State: The checkout is clean at c008a9c. Compiler work targets verified direct-function shared-borrow contracts and caller-origin substitution with the documented all-input lifetime bound. Runtime work targets bounded child admission and waiting joins on one worker. Root owns native examples/coverage and both trackers; independent review audits contract soundness.
- Validation: The previous milestone passed all 14 repository checks. New function/child behavior is not yet implemented or validated. Exclusive loans, captures, indirect calls, generated cleanup and DWARF remain separate work.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Define symbolic input origins versus conservative lifetime bounds, implement verified calls/results, design bounded parent-child waiting states, and add accepted/E302/E303 plus one-worker child-borrow execution coverage.

### 2026-09-06 — Complete reference-union and scheduler handoff

- Completed: compiler implementation is 75785b7, native example/coverage is 4ef3268, and runtime/tooling is d3e41aa. Both current STATUS files record exact behavior, remaining gaps, validation and ordered continuation work; prior checkpoints remain preserved.
- Validation: all 14 repository checks passed, including 136 Rust tests, 32 Python regressions, 844 links and every runtime debug/release/sanitized profile. The optimized example has exact stdout; all 588 release oracle cases pass (356 accepted, 232 E302). Final tracker links and whitespace pass.
- Blockers: none for this milestone. Conformance remains 9 passed, 14 unsupported, 0 failed. Function borrow contracts, exclusive ownership, generated cleanup, structured child joins/cancellation and DWARF remain pending.
- Next steps: implement verified all-input function borrow contracts and caller-origin substitution; extend ownership read/reborrow/cleanup edges; add structured child admission and waiting joins to the scheduler; continue the module/library foundation. Keep STATUS current and add a checkpoint after each logical step.

### 2026-09-06 — Refresh reference-union and scheduling handoff

- State: Current snapshots now describe compiler 75785b7, native coverage 4ef3268 and runtime/tooling d3e41aa. Completed union/scheduler priorities were replaced by function contracts, ownership cleanup, structured child lifetimes and modules. Historical checkpoints remain intact.
- Validation: All 14 combined checks, optimized optional-borrows output and all 588 release oracle cases passed. Final tracker link/whitespace verification is next.
- Blockers: no active workers, incomplete code or failing checks; full-language and structured-task support remain pending.
- Next steps: Verify the final tracker links/whitespace, commit this handoff, then resume the ordered function-contract, ownership and runtime work in STATUS.

### 2026-09-06 — Commit optional-borrow native coverage

- State: Native union/reference coverage, optional-borrows.mwy and compiler usage docs are committed after implementation 75785b7 and runtime d3e41aa. Coverage includes absence, retagging, nested same-name tags, mutable snapshots, restart, strict equality, literal widths and discarded constructor fields.
- Validation: All 46 native groups pass in both profiles; optimized example and 588 release oracle cases pass. Staged whitespace checks pass. Only current STATUS and step-log handoff updates remain.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Rewrite the current root/compiler snapshots with final commits, precise remaining boundaries and ordered next steps; validate links/whitespace and commit the final handoff.

### 2026-09-06 — Commit reference-union compiler implementation

- State: Compiler reference unions, scoped active-variant proofs, contextual record constructors, strict equality regressions and weighted analysis bounds are committed as 75785b7. The two prior unsupported tests now cover mutable carriers, keeping the implementation slice coherent. Runtime/tooling is d3e41aa.
- Validation: All 14 combined checks, optimized example, 588-case release oracle and staged whitespace checks pass. Native example/coverage and final tracker snapshots remain to commit.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit the new native union/constructor regressions and optional-borrows example, then refresh both STATUS snapshots with exact commit IDs, validation and ordered function-contract/ownership/runtime next steps.

### 2026-09-06 — Verify optimized optional borrows and final union oracle

- State: The optimized compiler executes optional-borrows.mwy with exact output and passes all 588 independent union cases on a private binary snapshot. Final review finds no remaining blocker. Runtime/tooling is committed as d3e41aa; compiler, native coverage and trackers are ready for focused commits.
- Validation: Release oracle: 356 accepted, 232 expected E302, zero conservative or unexpected results. Exact example stdout: 7, 8, missing, 8, 42, text on separate lines, with empty stderr. All 14 combined checks, 844 links and whitespace pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit the compiler implementation with the two existing capability-boundary corrections, then native examples/coverage and the current root/compiler handoffs; finish with Git integrity and a clean worktree.

### 2026-09-06 — Pass the complete union and scheduler repository gate

- State: All 14 repository checks pass with reference unions, optional fields, contextual record constructors and bounded scheduling integrated. Strict union equality remains unchanged. No active source edits or failing checks remain; root is building the optimized compiler before final commit slices.
- Validation: 90 library plus 46 native Rust groups, 32 Python regressions, 844 local links, schemas/catalog, editors, formatting/Clippy/build and conformance pass. Runtime debug/release/ASan/UBSan/LSan include all 11 scheduler cases and exact fault probes. Conformance is 9 passed, 14 unsupported, 0 failed in both profiles.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run optional-borrows with the optimized compiler and exact stdout, rerun the 588-case oracle on that completed binary, then commit compiler implementation, native coverage and current STATUS/step-log handoffs separately.

### 2026-09-06 — Complete strict union semantics and start final integration

- State: Reference union implementation, constructor context, scoped loan proofs and weighted bounds are complete. Same-normalized-type equality is preserved; the root test now uses a correctly typed null union. All workers have finished code edits; root is starting the combined gate.
- Validation: 90 library groups, all 15 loan groups, the corrected equality source regression, formatting and all-target Clippy pass. The 588-case oracle and directed review pass on the earlier stable snapshot. Runtime d3e41aa passes all profiles; final integrated native/runtime verification is next.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run all 14 repository checks, build the optimized compiler and optional-borrows example, rerun the 588-case oracle on its stable binary, review final diffs, then commit compiler implementation, native coverage and current handoffs separately.

### 2026-09-06 — Preserve the documented union equality type rule

- State: Reference review confirms that union equality requires the same normalized union type. The new absent.view == null assertion was a test mistake, so root now compares with a typed empty nullable-reference union. The compiler owner is removing the tentative member-to-union equality coercion; mismatched operand types must retain E222.
- Validation: Constructor widths, defaults and discarded fields already execute correctly in both profiles. The tentative equality extension passed tests but conflicts with the reference and is not accepted. All 90 library groups and Clippy had passed before this correction; affected checks will be rerun.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Restore strict normalized union equality, pass the corrected native constructor test and explicit E222 regressions, then run the full combined gate and optimized example/oracle before compiler and handoff commits.

### 2026-09-06 — Expose nullable-union equality compatibility failure

- State: The record-union literal context correction passes the original native failure. Expanded constructor execution now exposes E222 for comparing a nullable reference field directly with null; compatible union/member equality needs checking before final validation.
- Validation: The new native test remains intact and fails at absent.view == null. Root is isolating width/default/discarded-field execution with a temporary predicate substitution so that later constructor defects are visible. Core borrow oracle and directed probes remain passing.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Fix union/member equality according to the reference while preserving full-record/scalar projection behavior, pass the unchanged native constructor group, then complete source/budget regressions, formatting/Clippy and the combined gate.

### 2026-09-06 — Fix union-record constructor context and check native widths

- State: The previously failing reference-union retagging native group now passes. The frontend keeps union member context while selecting a compatible completed record shape. Root added execution coverage for int64 and uint8 fields/primaries, nullable reference defaults and named fields discarded on restart.
- Validation: The targeted retagging group passes in debug/release. The new constructor execution group is running; 588 oracle and 10 directed semantic checks remain passing on the pre-context-fix snapshot. No known borrow-analysis blocker remains.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Pass the constructor execution group and all core/source tests, finish OWNERSHIP and formatting/Clippy, then run the complete repository gate and optimized compiler/example/oracle before focused commits.

### 2026-09-06 — Pass directed union lifetime and restart review

- State: The stable compiler snapshot passes all 10 directed semantic probes beyond the 588-case oracle: cross-iteration retained loans, fresh optional rebindings, inactive/active escapes, retagging, discarded results and tag-only inspection. Native frontend record-union literal context remains the only known blocker.
- Validation: Oracle remains 356 accepted and 232 expected E302 with no conservative or unexpected results. Directed checks all pass; accepted directed programs are being exercised natively in both profiles. Literal context correction must preserve unique numeric widths and reject ambiguous choices.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish frontend record-union context and source/budget regressions, rerun all native groups, then perform formatting/Clippy, the integrated repository gate and optimized example/oracle before focused compiler commits.

### 2026-09-06 — Pass all 588 optional-reference guard cases

- State: The independent oracle passes all 588 nullable-reference combinations on a private snapshot of the freshly built compiler. Scoped activity handles absence, payload copies, retagging, nested field access and tag-only inspection without conservative rejections in this matrix. Loan unit integration also passes.
- Validation: Oracle: 356 accepted, 232 expected E302, zero conservative or unexpected results. All 15 loan groups pass. Native 44/45 pass; the remaining frontend record-union literal context failure is still being fixed. Directed restart/retag review continues.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish the frontend union-record context correction and preserve literal widths, rerun native/library checks, complete directed review and proof-budget tests, then run final formatting/Clippy and combined verification.

### 2026-09-06 — Pass optional-reference native behavior and isolate record-union context failure

- State: The integrated compiler passes 44 of 45 native groups, including optional reference activity, nested discriminants, stale mutable snapshots, restart behavior, E302/E303 boundaries and the new example. One record-union literal is incorrectly receiving scalar context in the frontend and is being corrected.
- Validation: Failure: reference_union_retagging_keeps_member_and_field_origins reports E207 scalar result cannot contain field view in the branch record literal. This is a frontend contextual typing failure, not a borrow rejection. A fresh standalone compiler build is being prepared for the independent oracle.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Fix expected-union record literal context without losing numeric widths or record shape, rerun that native group, execute the 588-case oracle and directed probes, then complete final source/budget tests and repository gate.

### 2026-09-06 — Compile union value and loan analysis together

- State: The union value/origin pass and loan CFG now compile together. Variant identities, null/default activity, nested tag relations, mutable-read invalidation, scoped assumptions and demand-preserving coercions are integrated. Root is running all native groups while workers finish focused source regressions.
- Validation: cargo check --lib passes. Compiler execution and oracle validation are now starting; no result is claimed yet. Runtime d3e41aa remains fully validated.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Resolve any native/source failures, inspect budget and restart boundaries, build a fresh standalone compiler for the 588-case oracle, then run final formatting/Clippy and the combined repository gate.

### 2026-09-06 — Review integrated union value semantics

- State: Early semantic review finds member-type retagging, outer-conditioned nested tags and fresh mutable-read activity in place. Scoped proof accumulation replaces repeated live-local scans. Loan graph helpers are still being implemented separately, so no complete compiler behavior claim is made.
- Validation: Isolated resource rejection probes pass; runtime d3e41aa is independently validated. New accepted/rejected native and 588 oracle cases are ready but have not yet run on the integrated compiler.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish and compile loan assume/convert/inspect helpers, run targeted native and source regressions, then audit the fresh executable and perform the full combined gate before compiler commits.

### 2026-09-06 — Preserve construction loans under tag-only inspection

- State: Root added a focused E302 case where a discriminant test consumes a freshly constructed nullable reference block that writes its owner after emission. Inspecting only a tag may skip reading an existing payload, but must preserve construction effects and retained result-slot liveness.
- Validation: Compiler value and graph integration are in progress in separate files. Isolated proof-budget probes and all committed runtime checks pass; new compiler execution remains pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete tag-only inspection without bypassing block construction, compile the integrated passes, then run native library/output/budget tests and the independent semantic oracle.

### 2026-09-06 — Parallelize union value and loan-graph integration

- State: Compiler interfaces are now stable enough for split ownership: borrow_design owns borrow_value.rs, borrow.rs, check.rs, flow.rs and OWNERSHIP; unwind_runtime owns loans.rs integration; storage_review audits semantics. Root owns native/example/docs/tracking and commits. Runtime remains committed as d3e41aa.
- Validation: Isolated resource probes reject wide fanout, deep path copies and growing merges within explicit budgets; the proof-work counter saturates. Review also identified repeated live-local assumption scans for charging or scoped caching. The complete compiler does not yet compile because loan helpers are pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish assume/convert/discriminant-inspection helpers and value proof linking, run compiler library/native checks, then execute the 588-case union oracle and directed restart/mutable-snapshot probes before final integration.

### 2026-09-06 — Reject stale mutable-union snapshots in copied predicates

- State: Semantic review found that copying a mutated ordinary union or record must not reuse its initializer activity snapshot. That could falsely prove an optional borrow absent and hide E302. Root added rejection cases for both shapes; the compiler owner will use fresh unknown activity for mutable storage reads.
- Validation: The new value/borrow interfaces are being integrated and are not yet build-qualified. Runtime is committed and validated. Existing native fixtures remain unchanged except coherent unsupported-boundary updates.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish mutable-read invalidation and scoped tag assumptions, compile the analysis interfaces, then execute targeted E302/E303/native cases and the 588-case oracle before the combined gate.

### 2026-09-06 — Preserve loop resets and bound union fact construction

- State: Runtime/tooling is committed as d3e41aa. Root added native loop cases for an outer nullable loan used on a later iteration and newly constructed optional loans across a toggled predicate. Independent resource review found a frontier-limit overshoot in the new union value helper; the compiler owner is fixing it during integration.
- Validation: Runtime profiles and tool checks pass. New compiler code is incomplete; native union and oracle checks remain pending. Current STATUS distinguishes the previous completed milestone from active work so an interrupted handoff is unambiguous.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Bound variant/path fanout before allocation, complete scoped tag/origin integration, then run compiler native and budget regressions, the 588-case oracle and combined repository verification.

### 2026-09-06 — Commit bounded scheduler and verification integration

- State: The independently validated scheduler and tooling are committed, including fixed admission, round-robin selection, bounded pumping, cleanup-before-settlement and explicit join retry. Compiler union value/variant analysis is now being implemented; root owns new native coverage and current handoffs.
- Validation: All runtime profiles pass with 11 scheduler cases and exact kernel/fatal probes; 12 runtime and 16 tooling Python tests pass. All 844 local links and staged whitespace checks pass. Compiler native union tests and full combined gate are still pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete scoped union origins/tag assumptions and coercion mapping, execute all 45 native groups and 588 oracle cases on a fresh compiler, then run the full repository gate and split compiler/coverage/handoff commits.

### 2026-09-06 — Pass scheduler sanitizer and ownership review

- State: Bounded scheduler admission, pumping, cleanup and explicit join pass all runtime profiles. Runtime rules and docs now state callback/ticket/storage lifetime, worker affinity, transition-only pump bounds and remaining structured-task limitations. Root review found no outstanding lifecycle defect.
- Validation: 11 scheduler cases plus real admission refusal and exact fatal cleanup pass in debug/release/ASan/UBSan/LSan; existing cleanup/stack/context checks and ASan expired-local evidence remain green. Compiler union implementation is still pending, so the combined gate has not run for this milestone.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finalize runtime Python/link/whitespace checks and commit that coherent slice; continue union origin/tag integration and scoped loop facts, then run native compiler cases, oracle and the full combined gate.

### 2026-09-06 — Cover nested discriminants without contradictory assumptions

- State: Independent review identified that same-named optional fields in different outer record variants can share a frontend tag domain. Their construction relations must be conditioned on the outer variant; unconditional relations could hide a live-loan conflict. Root added accepted and E302 native coverage for this exact shape.
- Validation: The 588-case oracle is prepared but not yet executed. Native compiler coverage now has 45 groups awaiting the union implementation. Scheduler debug/release and Python checks remain passing; sanitizer validation is progressing.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Condition nested tag relations on active outer variants, preserve those conditions through retagging and restart, then run all native groups and the independent oracle; finish scheduler sanitizer and documentation checks.

### 2026-09-06 — Pass bounded scheduler debug and release checks

- State: The fixed-slot scheduler passes debug/release with round-robin selection, explicit settlement/join and cleanup that can suspend. Root review found no lifecycle defect; documentation will make borrowed callback storage, nonreentrant controls and transition-budget limits explicit.
- Validation: 11 scheduler cases, real kernel ENOMEM admission/joined failure and exact fatal task-cleanup evidence pass in both profiles; all prior runtime suites pass. Twelve runtime Python regressions and 16 tooling tests pass. Sanitizers and compiler union implementation remain pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run scheduler ASan/UBSan/LSan, finish runtime docs, implement scoped union activity facts and retagging, then run new native compiler cases and the 588-case oracle before final integration.

### 2026-09-06 — Integrate bounded scheduler verification scope

- State: Scheduler core and 11 native cases now cover fixed-slot admission, bounded pumping, round-robin selection, settlement, cleanup suspension, stale/foreign tickets and explicit join retry. Tool descriptions include bounded scheduling while keeping structured child cancellation/join and DWARF separate.
- Validation: 16 tooling regressions pass. Runtime scheduler compilation/profile checks are next; compiler union implementation remains in progress. The independent union oracle is ready with 588 cases and has not run against an unfinished build.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run runtime debug/release/sanitizers with strict failed-admission and cleanup evidence, complete compiler variant facts, then execute 44 native groups and the 588-case union oracle on a freshly completed compiler build.

### 2026-09-06 — Keep variant assumptions local to binding lifetimes

- State: Compiler design now attaches constructor/tag relations at immutable binding edges, so restarts discard iteration-specific assumptions. Root corrected the discarded optional-field output case to use a type predicate in matcher position; an expression ascription is not a boolean test.
- Validation: Reference and parser inspection confirm suffix context determines predicate versus ascription. Existing source cases remain unchanged; new native tests still await union implementation. Round-robin selector is complete and runtime construction is progressing.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement scoped variant assumptions and coercion paths, then check all new native cases; verify scheduler bounded pumping, stale tickets, retained allocation failures and cleanup-before-join.

### 2026-09-06 — Implement round-robin task selection

- State: Root filled runtime/src/task_policy.cpp after allowing time for the optional contribution. It scans the fixed span from the current cursor, wraps safely, skips nonrunnable slots and returns the span size when empty. The temporary placeholder and purpose comment were removed.
- Validation: Selection code is ready for native lifecycle and bounded-pump tests. No scheduler execution claim is made yet; runtime implementation and compiler union analysis are still in progress.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish scheduler admission, cleanup and explicit join, then verify selection/worker/stale-ticket boundaries; complete compiler union activity and run accepted/rejected native cases plus the independent oracle.

### 2026-09-06 — Connect union activity to discriminant proofs

- State: Compiler design now links immutable union activity to later discriminant tests, including null defaults and variant retagging by member identity. Payload-free type tests will not read borrow payloads. README and native examples describe the intended supported boundary; implementation and validation are still pending.
- Validation: Independent review identified the required correlation between emission guards and later type-test guards; global assumptions must not leak across restart or mutable-predicate invalidation. Baseline compiler tests remain green.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement bounded variant facts and scoped invariants, verify optional projections and loop resets, finish scheduler lifecycle and task-selection policy, then run native and oracle checks before claiming the feature supported.

### 2026-09-06 — Prepare bounded scheduler policy and refine discarded-slot coverage

- State: The runtime now has a fixed-slot scheduler interface and an optional 5–10-line selection-policy contribution at runtime/src/task_policy.cpp:5. Its placeholder returns no runnable work until filled. Independent review corrected the discarded-only field test to use an explicit optional-field annotation, matching reference inference rules.
- Validation: Baseline compiler tests pass; new union cases await implementation. Scheduler native execution has not started because admission/lifecycle code and the selection helper are incomplete. The language leaves task order unspecified; round-robin is the proposed implementation policy.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Allow time for the optional selection contribution while completing union origins and scheduler lifecycle; fill the default policy if declined/no response, then run native suites and the independent 500-case union oracle.

### 2026-09-06 — Specify nullable borrow execution and rejection coverage

- State: Root added optional-borrows.mwy and native cases for absent references, optional nested fields, reference and record union retagging, variant-specific owner writes, discarded result resets, live copies/equality and E303 escapes. Compiler and runtime implementations are in progress.
- Validation: Baseline 80 library and 39 native groups pass. New cases are specifications awaiting active-variant analysis; scheduler scope and the optional selection-policy helper are still being prepared.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete union guard/origin propagation and review retagging by member identity; prepare scheduler admission/pump lifecycle, then execute the new native coverage and fill the bounded selection helper.

### 2026-09-06 — Begin reference unions and bounded scheduling

- State: The checkout is clean at 4273496. Compiler work targets immutable reference-containing unions, optional fields, coercions and guarded extraction; runtime work targets bounded worker-owned scheduling over pinned contexts. Root owns native coverage and both trackers; independent review covers variant/lifetime hazards.
- Validation: The previous milestone passed all 14 repository checks. New compiler/runtime behavior is not yet implemented or validated. Function contracts, exclusive loans, full structured child trees and DWARF remain separate work.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement active-variant component origins and scheduler admission/lifecycle; prepare optional task-selection policy contribution, add native accepted/E302/E303 cases, and verify both profiles and sanitizer transitions.

### 2026-09-06 — Complete reference-record and context handoff

- Completed: component borrow implementation is 0b24669, native coverage/example is 7816524, and pinned runtime/tooling is 3dbed45. Both STATUS snapshots contain current behavior, explicit gaps, evidence and ordered next steps; all prior checkpoints are preserved.
- Validation: all 14 repository checks passed: 119 Rust groups, 31 Python tests, 841 links, native profiles with exact fatal/guard evidence and ASan/UBSan/LSan. Optimized example output and all 300 component oracle cases passed. Final tracker links and whitespace pass; unchanged upstream fcontext.hpp retains its documented blank-at-EOF exception.
- Blockers: none for this milestone. Conformance remains 9 passed, 14 unsupported, 0 failed; reference unions/function contracts, exclusive ownership, structured scheduling and DWARF remain unimplemented.
- Next steps: carry component origins through reference unions and verified function contracts; add ownership read/reborrow/cleanup edges; build bounded scheduling and structured child joins over contexts; continue module/library implementation. Follow each STATUS Next steps section and keep unsupported behavior explicit.

### 2026-09-06 — Refresh component and context handoff

- State: Current snapshots now describe immutable reference records, full-shape equality, pinned runtime contexts and the exact remaining capability boundaries. Completed priorities were replaced by concrete union/function, ownership, scheduling/unwind and module work. Historical checkpoints remain intact.
- Validation: Implementation 0b24669, native coverage 7816524 and runtime/tooling 3dbed45 have passed the combined gate, optimized example and component oracle. Final tracker link/whitespace checks are next.
- Blockers: no active workers or incomplete code; full-language and structured-task support remain pending.
- Next steps: Verify current tracker links and whitespace, commit the handoff, then resume with component origins through reference unions and verified function contracts alongside bounded runtime scheduling.

### 2026-09-06 — Commit pinned native context integration

- State: Compiler implementation and native coverage are committed as 0b24669 and 7816524. Runtime context lifecycle, upstream pin/license, register and sanitizer probes, runtime rules and tool descriptions are now committed. Only current handoff and step logs remain.
- Validation: All 14 combined checks, optimized borrowed-record execution and 300 component oracle cases pass. Git checks passed with the documented unchanged upstream blank-at-eof exception.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Rewrite current root/compiler STATUS to replace completed priorities, record the three implementation commits and remaining boundaries, check local links and tracker whitespace, then commit the handoff.

### 2026-09-06 — Preserve upstream bytes during runtime staging

- State: Compiler implementation is 0b24669 and native coverage/example is 7816524. Pinned runtime files are staged. Git flags one upstream trailing blank line in the unchanged fcontext.hpp ABI reference; preserve it so source bytes and recorded SHA-256 remain exact.
- Validation: Staged whitespace checks pass for every other file. The ABI reference passes with only blank-at-eof disabled; runtime checksum validation and all debug/release/sanitizer cases pass. This is a vendored-source formatting exception, not a runtime failure.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit runtime contexts and tooling with the upstream file intact; refresh both STATUS snapshots with final results, known limits and ordered next steps, then commit the handoff.

### 2026-09-06 — Commit borrowed-record native coverage

- State: Reference-record native coverage, borrowed-records.mwy and compiler usage documentation are committed after implementation 0b24669. Coverage protects projections versus full copies, guarded fields, retained results, equality and explicit unsupported contracts.
- Validation: 39 native groups pass in both profiles. The optimized compiler/example and all 300 guarded component cases pass; staged whitespace checks pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit pinned runtime context lifecycle, exact upstream provenance and verification integration; then refresh current STATUS files with final commit IDs and precise continuation work.

### 2026-09-06 — Commit component borrow analysis

- State: Compiler component origins, per-leaf liveness, record/scalar equality inference and ownership design are committed. The existing unsupported test now covers mutable reference records so this implementation commit remains coherent.
- Validation: All 14 combined checks, optimized example, 300-case component oracle and staged whitespace checks passed. Native coverage/example, runtime/tooling and trackers remain to commit.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit native reference-record regressions and example next; then commit pinned runtime contexts with provenance/tooling and finish the current handoff.

### 2026-09-06 — Verify optimized borrowed-record execution

- State: The optimized compiler builds borrowed-records.mwy with exact output and passes the independent 300-case component oracle. Final code, native coverage, runtime provenance and documentation review found no remaining blocker.
- Validation: Release oracle: 204 accepted, 96 expected E302, zero conservative or unexpected results. Exact example output: 11, 44, 33, 44, value, 0, 1, 2, 3 on separate lines. The combined 14-check gate and whitespace checks passed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit component implementation with the existing capability-boundary correction, then native example/coverage, pinned runtime/tooling and final handoffs as separate focused slices.

### 2026-09-06 — Pass the combined compiler and context gate

- State: All 14 repository checks pass with immutable reference records and pinned native contexts integrated into verification. No compiler or runtime code blocker remains.
- Validation: 80 library plus 39 native Rust groups, 31 Python regressions, 841 local links, schemas/catalog, editors, formatting and Clippy pass. Runtime passes 14 cleanup, 10 stack and 10 context cases per debug/release/sanitized profile with exact fault probes; ASan catches the expired fiber local. Conformance remains 9 passed, 14 unsupported, 0 failed in both profiles.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Build the optimized compiler, execute borrowed-records with exact stdout, rerun the 300-case component oracle on that completed binary, then review and split commits and finalize both STATUS handoffs.

### 2026-09-06 — Complete component inference and pinned context review

- State: Immutable reference records and record/scalar equality inference are complete. Independent review found no remaining component blocker. Runtime provenance, lifecycle and sanitizer-hook documentation are complete; the combined repository gate is next.
- Validation: 80 compiler library groups and all-target Clippy pass; 7 independent compiler probes pass in both profiles. Runtime debug/release/sanitizers, 11 Python regressions and 841 local links pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the combined repository gate, optimized compiler example and final guarded component oracle; then inspect and split coherent commits before refreshing the handoff.

### 2026-09-06 — Validate pinned context execution and review record components

- State: Pinned context debug/release/sanitizer runs now pass. Component code review is clean apart from equality-context inference; that fix is switching to the existing partial composition checker rather than syntax heuristics. Compiler README now documents immutable reference fields, copies, projections and explicit remaining boundaries.
- Validation: Runtime passes 10 context cases plus fatal cleanup after resume per profile; ASan detects the returned-fiber-local negative probe. Existing cleanup/stack checks remain green. Compiler guarded oracle passes 300 cases; final equality/native recheck is pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete semantic equality inference, provenance validation and runtime docs; then run compiler/native checks and the integrated repository gate before focused commits.

### 2026-09-06 — Preserve scalar comparisons while fixing record equality

- State: The equality correction now compares full returned records and catches RHS mutation conflicts, but independent review found it over-constrains scalar block operands to the record shape. Native coverage now checks aggregate-versus-scalar block comparison in both directions after an unrelated reference owner changes.
- Validation: The remaining compiler blocker is E204 for packet == a scalar-producing block; the language requires primary projection in that case. The 300 component cases and 16 tooling regressions pass. Runtime context profile validation is ongoing.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Infer equality operand shape without discarding record fields or imposing them on scalar blocks; retain contextual numeric widths, then rerun native suites and context profiles.

### 2026-09-06 — Expose full-record equality projection defect

- State: Independent review reproduced a real equality defect: a block returning an existing record could be reduced to its scalar primary, producing true for unequal reference fields and hiding a live-loan conflict. Native coverage now tests returned-record equality and both literal/returned-record RHS mutations.
- Validation: The 300-case component oracle passes. The new equality probes demonstrate the defect in the current build; frontend context correction is pending. Pinned runtime context profile checks are in progress.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Preserve full record shape for equality contexts, verify false/true pointer-field comparisons and E302 across RHS effects, then finish context sanitizer and provenance checks.

### 2026-09-06 — Pass the guarded component oracle and assemble context probes

- State: The completed compiler passes all 300 guarded component/copy oracle cases. Root added native interpolation of a scalar primary with unrelated reference fields. Runtime now has explicit context lifecycle tests, multi-context host alternation, nested yields, register/FP-state probes and suspension during explicit cleanup.
- Validation: Record oracle: 204 accepted, 96 expected E302, no conservative rejections or unsafe acceptances. Independent review matched 13 of 14 probes; the remaining result is the known record-equality contextual E207. Runtime profile checks and fiber-lifetime negative evidence are pending.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete record equality typing and run all native groups; validate pinned contexts in debug/release/sanitized profiles with exact negative-probe evidence and checksum-verified upstream sources.

### 2026-09-06 — Run borrowed-record integration checks

- State: Component-aware record origins and CFG leaves are integrated. Native checks pass 37 of 38 groups, including projection-only liveness, nested/primary records, guarded slots, named control flow and E303 escapes. One aggregate-equality operand is incorrectly given a scalar expected type and is being corrected.
- Validation: The failing equality case reports E207 before loan analysis instead of the expected E302. The first oracle invocation raced the compiler build and used an older executable; its results were discarded and the completed build is now being audited. Runtime context lifecycle tests are still being assembled.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Fix contextual record equality, rerun native and the 300-case oracle on the completed binary, then verify and document the pinned context lifecycle and sanitizer transitions.

### 2026-09-06 — Integrate component identities and pin context sources

- State: Compiler origins now carry record component identities while CFG adaptation is in progress. Native coverage includes safe projection from a local record versus an invalid whole-record escape, and an independent 300-case guarded component/copy oracle is ready. Runtime sources are pinned to Boost.Context revision 6ff80e0575133c6d0b704e03bbb956a0c3b9551a with upstream license and hashes.
- Validation: The focused native invocation reached an incomplete interface migration: loans.rs still lacked the new Origin.component initializer. This is pending implementation, not a test result. Baseline 106 Rust/native checks passed before edits. Runtime lifecycle and fiber-hook validation has not run yet.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish per-component CFG adaptation, then run native and oracle checks; complete explicit context initialization/resume/yield/release, thread checks and sanitizer handoff tests.

### 2026-09-06 — Specify borrowed record execution and rejection coverage

- State: Root added borrowed-records.mwy and native cases for named/primary/nested components, record copies, scalar-only projections, guarded origins, named restart/leave and component escapes. Compiler implementation and pinned-context prototype are in progress.
- Validation: Baseline 72 library and 34 native tests pass; conformance is 9 passed, 14 unsupported, 0 failed in both profiles. Review requires whole-record copies to retain all references while direct field projections retain only their selected leaves.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run new accepted/E302/E303 cases once component-aware origins and CFG are integrated; verify context lifetime, worker pinning and sanitizer handoff against pinned upstream primitives.

### 2026-09-06 — Begin borrowed records and pinned context work

- State: Compiler work now targets immutable records carrying shared references, with per-component origins and final-use liveness. The runtime worker is implementing the planned pinned context wrapper on guarded stacks; independent review covers component lifetime/copy hazards. Root owns native examples, coverage and both trackers.
- Validation: The checkout is clean at 4e5ed15 after initial transient tracker edits disappeared before this task wrote files. Baseline compiler/native and conformance checks are running. Reference-bearing unions, mutable aggregate bindings, function contracts and exclusive loans remain separate compiler work.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement record leaf origins/projection liveness and the private pinned context lifecycle; verify copied versus projected fields, all escaping origins, worker pinning, register preservation and sanitizer switching hooks.

### 2026-09-06 — Complete loan-liveness and guarded-stack handoff

- Completed: compiler implementation is `94b9160`, native coverage/example is
  `e557d63`, and guarded runtime/tooling is `7a5d286`. Current status now records
  supported behavior, precise limits, verification and ordered continuation work.
- Validation: all 14 combined checks pass, including 106 Rust and 28 Python tests,
  native/runtime profiles, 836 local links and the optimized compiler example.
  The release guard audit passes all 196 combinations. Final diff checks pass.
- Blockers: no active workers, incomplete code or failing checks. Conformance still
  has 14 unsupported cases; task switching and DWARF unwinding are not implemented.
- Next steps: add exclusive access/read/reborrow analysis and aggregate/function
  borrow contracts; connect guarded allocation to the pinned context wrapper,
  then advance structured cleanup, modules and real library sources.

### 2026-09-06 — Commit guarded runtime and verification integration

- State: Guarded StackMemory allocation, native/Python coverage, runtime rules and repository verification descriptions are committed as 7a5d286. Compiler implementation and coverage are 94b9160 and e557d63. Only final tracker edits remain uncommitted.
- Validation: Combined verification and every runtime profile passed before commit, including strict fault evidence and sanitizer checks. Runtime commit staged whitespace checks passed; no implementation edits or active workers remain.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finalize current status with the three commit IDs, exact validation and remaining compiler/runtime boundaries; check links and whitespace, then commit the documentation handoff.

### 2026-09-06 — Commit native loan and divergence coverage

- State: Native borrowing and short-circuit regression coverage, borrow-liveness.mwy and compiler usage documentation are committed as e557d63, following implementation 94b9160. Runtime/tooling and final tracker updates remain.
- Validation: All 34 native groups passed in debug/release; the optimized compiler example and 196 guard combinations passed. Native commit staged whitespace checks passed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit bounded guarded stack allocation with runtime harness and verification descriptions; finalize current limits, evidence and ordered next steps in both status files.

### 2026-09-06 — Commit shared-loan implementation

- State: Compiler CFG, guarded origins, last-use checking, short-circuit lowering and analysis bounds are committed as 94b9160. The existing unsupported-boundary test changed with the implementation so the commit does not retain a stale mutable-root rejection.
- Validation: The integrated 14-check gate, optimized example, 196-case release guard audit and staged whitespace checks passed before commit. Only native/example docs, runtime/tooling and tracker slices remain uncommitted.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit native regression coverage and borrow-liveness example, then guarded runtime integration, then refresh and commit the final handoff.

### 2026-09-06 — Verify optimized compiler and stage a coherent implementation slice

- State: The release compiler and borrow-liveness executable pass with exact output; the final release compiler also passes the 196-case guard oracle. The first staged slice contains CFG/origin/frontend/backend changes, ownership design and the existing mutable-borrow capability-test correction; new native coverage remains separate.
- Validation: Release output is exactly 41, 42, 7, 9, 0, 1, 2, 3 on separate lines. Initial probe incorrectly used run --output and hit the documented CLI restriction; corrected to build then execute. Release guard oracle: 114 accepts, 82 expected E302, no other results. Staged whitespace checks pass.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Commit the staged compiler slice, then native examples/coverage, guarded runtime/tooling and current handoff; finish with local link and Git integrity checks.

### 2026-09-06 — Pass the combined compiler and runtime gate

- State: All 14 repository checks pass on the integrated implementation. No active worker edits or failing test remains. Root is building the release compiler and preparing dependency-ordered commits; the full language and task-switching milestones remain incomplete.
- Validation: Combined gate: 72 library plus 34 native Rust tests, 28 Python tests, 836 local links, schemas/catalog, Vim/Neovim, formatting/Clippy/build and conformance pass. Each runtime profile passes 14 cleanup cases plus 2 fatal probes, 10 stack cases plus kernel ENOMEM and 2 exact guard faults. Conformance remains 9 passed, 14 unsupported, 0 failed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Verify the optimized compiler against borrow-liveness.mwy, update current handoff/remaining priorities, then commit implementation, native coverage, runtime integration and trackers in separate focused slices.

### 2026-09-06 — Bound both origin caches and start final integration

- State: Compiler hardening is complete: both the CFG origin store and preceding borrow-fact cache have cumulative limits, and final overlap scans consume the analysis work budget. Compiler/runtime workers have finished edits; root is running final integration and preparing focused commits.
- Validation: Dense liveness and guarded-origin alias stress sources reject with B001 at the intended limits. Formatting and all-target Clippy pass. All 34 native groups, the 196-case guard oracle, independent probes and runtime profiles passed before this final integration run.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the combined repository gate with LSan-compatible access and the release borrow-liveness example; record exact results, inspect staged slices, then commit compiler, coverage, runtime/tooling and handoff separately.

### 2026-09-06 — Review loan semantics and bound origin storage

- State: Independent review found no semantic blocker in CFG temporaries, result slots, guarded writes, loop resets or backend Never termination. It identified an unbounded aggregate of cloned origin entries and uncharged final overlap scans; the compiler worker is adding explicit storage/work accounting before the final gate.
- Validation: All 34 native groups and 196 guard combinations pass; 10 extra independent source probes match acceptance/E302 expectations. Formatting and Clippy pass. Dense 512-reference input reports B001 budget exhaustion; the additional origin accounting is not yet qualified.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish origin-storage and overlap-work bounds, rerun affected compiler checks, then execute the combined 14-check gate and release example before focused commits.

### 2026-09-06 — Pass last-use execution and short-circuit regression coverage

- State: All 34 native test groups now pass. The short-circuit fix accepts nonreturning operands and emits valid unreachable block endings without running skipped effects. Runtime and tooling edits are complete; compiler source budget coverage, ownership documentation and final independent review are finishing.
- Validation: Native execution passes in debug/release, including alias last-use writes, guarded non-overlap, retained result conflicts, equality temporaries, loop backedges, skipped blocks and panic/leave operands. The 196-case guard oracle also passes. Full integration gate has not run yet.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish loan-analysis budget regression and formatting/Clippy; run the 14-check combined gate with sanitizer access, review the complete diff and create focused local commits.

### 2026-09-06 — Verify guard matrix and finish stack-allocation handoff

- State: The runtime implementation and documentation are ready for integration with no active runtime edits. The independent compiler oracle completed all 196 combinations of origin selection, conditional writes and conditional uses. Compiler short-circuit correction and final CFG review remain active.
- Validation: Guard oracle: 114 accepted programs and 82 expected E302 rejections, zero conservative rejections and zero unsafe acceptances. Runtime profile checks and 8 Python regressions pass; runtime handoff reports 836 valid local links. One compiler native short-circuit case still awaits the fix.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Complete the compiler correction and independent review; run full formatting, lint, native/runtime/conformance gate, then split compiler implementation, coverage, runtime integration and tracker commits.

### 2026-09-06 — Run integrated native and guarded-stack checks

- State: The CFG implementation now passes 32 of 33 native test groups, including live-loan rejections, guarded origins, named control flow and the new example. One accepted short-circuit case exposed an existing Bool/Never typing mismatch and is assigned to the compiler worker. Runtime guarded stacks passed their first complete profile run.
- Validation: Runtime: 10 allocation cases, real ENOMEM and 2 exact guard faults pass per debug/release/sanitized profile, alongside 14 cleanup cases and 2 fatal probes; 8 Python regressions pass. Compiler failure: false && a block with effects is typed Bool versus Never (E222). Guard oracle and code review are in progress.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Fix short-circuit typing without evaluating skipped effects; finish the guard oracle and independent CFG/runtime review, then run the combined repository gate.

### 2026-09-06 — Implement guarded stack ownership and review loop conflicts

- State: Runtime now has a noncopyable StackMemory owner and ten native allocation cases, including injected OS failures and explicit cleanup integration. Isolated probes cover real allocation failure and both guard boundaries. Compiler native coverage adds cross-iteration conflicts whose source guards look disjoint; CFG integration is still in progress.
- Validation: Runtime harness/profile execution is pending. The existing implementation gate remains the baseline; new compiler acceptance tests still require the loans module to be completed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run guarded-stack debug/release/sanitizer profiles and their strict subprocess evidence checks; complete compiler CFG and verify that loop backedges cannot hide a live reference conflict.

### 2026-09-06 — Cover conditional origins and document final-use writes

- State: Native coverage now includes safe writes to unselected origins and disjoint use/write guards. The compiler README and example describe shared mutable-owner borrows, final-use RHS evaluation and whole-record overlap; these claims await the implementation gate.
- Validation: New mutable-root execution remains pending the CFG implementation. A temporary independent oracle is ready to audit 196 guarded origin/write/use combinations against both Boolean input values. Tooling regressions remain green.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Run the native cases and the guard oracle against the integrated compiler; review loop fixed points and held reference operands before accepting the milestone, then validate runtime and combined checks.

### 2026-09-06 — Add native last-use regression cases

- State: Root added the borrow-liveness example and native cases for mutable local/record roots, aliases, operand evaluation, result slots, named leave and loop backedges. Tooling descriptions now include the pending guarded stack checks. Compiler and runtime implementation workers remain active.
- Validation: The first new native test fails at the existing mutable-root B001 boundary as expected before integration. All 16 tooling regressions pass. Baseline 93 Rust/native tests and 9 supported conformance cases passed before these edits.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Integrate CFG analysis, rerun new native cases and correct real failures; qualify bounded stack allocation and guard faults in all profiles, then run the combined gate.

### 2026-09-06 — Define last-use and stack-allocation boundaries

- State: Compiler work will track reference temporaries and result slots as well as local aliases, with bounded CFG liveness and scoped leave/restart edges. Runtime will add explicit fallible Linux stack mappings with two guard pages; context switching remains pending. Root is adding native accepted/rejected cases.
- Validation: Baseline cargo tests pass: 64 library and 29 native. Conformance passes 9 cases with 14 unsupported and 0 failures in debug/release. Independent review identified reference temporaries across RHS effects as an essential conflict case.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Implement CFG and guarded stack mappings; exercise final-use assignments, retained emissions, aliases, restarted loops and reference equality operand lifetimes, then run focused suites.

### 2026-09-06 — Begin loan liveness and bounded stack work

- State: Compiler CFG and backwards loan liveness are assigned to borrow_design; root owns native coverage and both trackers. Runtime bounded stack allocation is assigned to unwind_runtime; storage_review independently reviews compiler soundness. No new behavior is qualified yet.
- Validation: The tree began clean at 89767ef. Baseline 64 library tests pass; native tests are running. Reference memory and compiler/runtime plans were reviewed.
- Blockers: none requiring user input; unsupported implementation areas stay explicit.
- Next steps: Finish baseline checks; implement shared borrows of mutable roots with E302 conflicts and last-use acceptance, plus guarded native stacks. Verify aliases, result transfers, loops and assignment ordering before integration.

### 2026-09-06 — Verify tracker split and continuation rules

- Completed: current status and historical logs link to each other. Repository,
  compiler and runtime rules now keep snapshots and checkpoints separate, with
  one writer and explicit next steps after every logical work step.
- Validation: all 21 root and 43 compiler checkpoints and completed compiler
  checklists match the previous committed text exactly. The local link check
  passed 834 links in 85 Markdown files; diff whitespace checks passed.
  Staged checking caught an extra blank line at the compiler log ending; corrected.
  Code and runtime tests were not rerun for this documentation-only change.
- Blockers: none; independent review confirmed the handoff rules and retained
  compiler remaining-work anchor. Implementation limits are unchanged.
- Next steps: keep new checkpoints here and current priorities in STATUS;
  resume CFG loan liveness, aggregate/function borrows and runtime context work.

### 2026-09-06 — Separate current status from historical checkpoints

- Completed: inspected tracker boundaries and moved the existing step log intact
  to this file. Current status keeps the handoff, gaps, evidence and next steps.
  Completed compiler milestone checklists are preserved below the step log.
- Validation: extraction completed; preservation, link and whitespace checks pending.
- Blockers: none found; this is a documentation-only change.
- Next steps: verify every old checkpoint and checklist is preserved, check links
  and whitespace, update current status and commit the focused documentation change.

### 2026-09-06 — Complete guarded borrowing and runtime handoff

- Completed: compiler behavior/coverage is `5e7ad41`, runtime prototype is
  `e0987be`, and repository integration is `1ba3c65`. Refreshed supported behavior,
  validation, remaining work and continuation order; this is the final docs split.
- Validation: all 14 combined checks passed with sanitizers enabled outside ptrace;
  93 Rust and 25 Python tests pass, plus native cleanup and release example checks.
  Conformance remains 9 passed, 14 unsupported, 0 failed in both profiles.
  Final documentation links and diff whitespace checks passed.
- Blockers: no active workers, incomplete code or failing milestone check. Predicate
  reassignment remains conservative; full loan dataflow and stack unwinding remain
  future work rather than implied capabilities of this milestone.
- Next steps: implement CFG loan liveness and aggregate/function origin contracts;
  qualify runtime context/unwind interaction, then progress module/library loading.

### 2026-09-06 — Commit the independent native cleanup prototype

- Completed: `e0987be` adds runtime API, implementation, behavior/fatal tests,
  pinned-tool runner, documentation and working rules outside the compiler.
- Validation: 14 normal cases and 2 fatal subprocesses pass in debug, release
  and ASan/UBSan/LSan; 5 Python regressions and staged whitespace checks pass.
- Boundary: compiler linkage, task contexts and DWARF unwinding remain pending;
  payload lifetimes and cross-stack cleanup ordering are explicit caller contracts.
- Next steps: commit repository runtime verification, then complete tracker maps,
  validation evidence and the concrete CFG/context/library continuation plan.

### 2026-09-06 — Commit guarded reference results

- Completed: `5e7ad41` commits emission proof IDs, guarded origin propagation,
  source/native regressions and the borrowed-results example together. The changed
  ancestor-result expectation is included with its behavior change.
- Validation: staged whitespace checks passed; full compiler/runtime gate and
  release example already passed on these source changes.
- Next steps: commit the independent native cleanup prototype, then its repository
  verification integration and the final remaining-work handoff.

### 2026-09-06 — Verify the optimized compiler and prepare split commits

- Completed: the release compiler built and ran `examples/borrow-results.mwy`
  under the release output profile, printing `11`, `22`, `42`, `true` exactly.
- Validation: all changed code and new runtime files have been reviewed; final
  whitespace checks pass. No worker or unresolved implementation defect remains.
- Next steps: commit compiler behavior with its source tests/example, runtime
  prototype with its tests/rules, tooling integration, and final handoff separately.

### 2026-09-06 — Pass the full compiler and cleanup integration gate

- Completed: `python3 -B tools/verify.py --all` passed all 14 checks outside ptrace
  supervision, with sanitizer checks enabled and unchanged source.
- Validation: 64 library + 29 native Rust tests; 16 tooling + 5 runtime + 4 compiler
  Python tests; 14 cleanup cases and 2 fatal probes in each debug/release/sanitized
  profile. Formatting, Clippy, build, 829 links, schemas and both editors pass.
- Conformance: 9 passed, 14 explicitly unsupported, 0 failed in debug/release.
  These results do not qualify task stacks, DWARF unwinding or full v0.0.1.
- Next steps: release-smoke the borrowed-results example, inspect all changes,
  commit compiler/runtime/tooling independently and finalize the continuation plan.

### 2026-09-06 — Audit borrow proofs and rerun sanitizer gate with required access

- Validation: 10 targeted borrow probes matched expected outcomes; accepted cases
  executed in debug/release. Another 288 predicate combinations found no unsafe
  acceptance. Stable proofs matched exhaustive lifetime safety; reassigned
  predicates conservatively rejected 15 safe combinations after facts were lost.
- Environment failure: combined verification passed repository/editor/runtime
  runner checks and native debug/release, then LeakSanitizer failed under sandbox
  ptrace. The unchanged full command is rerunning outside that supervision.
- Next steps: finish the combined gate, retain the predicate-mutation proof limit
  in the handoff, run the release example and prepare focused commits.

### 2026-09-06 — Pass focused native checks and finish prototype review

- Completed: guarded references pass native selection/alias, discarded-result and
  corrected local-escape tests. Added the borrowed-results example and refreshed
  compiler/storage documentation. Tooling now checks runtime docs and `--runtime`.
- Validation: 16 tooling tests pass. Independent runtime review found no concrete
  defect; 14 cleanup cases plus 2 exact fatal probes pass in each debug/release/
  ASan/UBSan/LSan profile. Five runtime runner tests pass.
- Boundary: runtime transfer moves only cleanup obligations. Partial emissions
  need deferred transfer or cross-stack order metadata before compiler integration.
- Next steps: finish borrow proof edge review, run the complete combined gate,
  release-smoke the example, then split compiler/runtime/tooling/handoff commits.

### 2026-09-06 — Validate guarded origins and integrate runtime verification

- Completed: the new origin pass preserves all alternative roots and passes its
  eight focused source-test groups. Complementary reference results execute in
  both profiles. Added explicit `--runtime` and expanded `--all` verification.
- Failure: a native rejection fixture emitted into an inner block, accidentally
  producing a nullable outer result and E222. Corrected its named emission target
  so it exercises the intended retained local escape.
- Runtime validation: 13 cleanup cases and 2 fatal probes pass in debug/release
  and ASan/UBSan; 5 runner tests pass. LeakSanitizer required outside-sandbox
  execution because ptrace blocks its inspection. Independent review is active.
- Next steps: rerun the corrected native cases, verify tooling/runtime integration,
  review origin soundness and run the combined gate after remaining edits settle.

### 2026-09-06 — Connect emission proofs and add source regressions

- Completed: each HIR emission now receives a unique proof ID, including generated
  record components and unreachable writes. Block completions retain the original
  guard arena. Backend lowering ignores proof IDs; code generation is unchanged.
- Coverage: added native programs for complementary reference selections, aliases,
  named leave, restart-discarded locals, panic effects and retained local escapes.
- Validation: integration waits for the new guarded-origin pass. Tests have not
  yet run against the changed API; runtime behavior tests are also being built.
- Next steps: finish origin propagation, run focused compiler/native checks and
  integrate the runtime verification entrypoint once its checks pass.

### 2026-09-06 — Choose guarded result identities and cleanup boundaries

- Decision: give each HIR emission a unique ID; keep write/completion guards in
  private proof tables and pass the existing guard arena to borrow checking.
  Preserve every possible origin, filter discarded paths, and validate lifetimes
  only for completed bare-reference results. Aggregates/signatures remain B001.
- Runtime: caller-owned cleanup entries, checked scope/token generations and
  explicit reverse cleanup are implemented; behavior/sanitizer checks are pending.
  No C++ exception, task-stack or scheduler behavior is being claimed.
- Ownership: root owns HIR/frontend/native integration; borrow worker owns borrow.rs;
  runtime worker owns runtime/. No new behavior has passed its full gate yet.
- Next steps: connect proof identities, propagate guarded origins, add source
  regressions and verify runtime cleanup edges before integration.

### 2026-09-06 — Resume guarded borrows and native cleanup work

- Completed: confirmed a clean tree and read working rules, current handoffs,
  ownership boundaries and compiler architecture. Prior milestone checks are the
  baseline; new behavior is not yet verified.
- Active work: root owns compiler integration and both STATUS files; a reviewer
  is tracing guarded borrow-result dataflow. A separate worker owns `runtime/`
  cleanup/unwind prototyping outside the compiler.
- Next steps: settle the guard metadata and result-origin design, implement safe
  immutable block-result transfers, and validate an explicit native cleanup ABI.

### 2026-09-06 — Complete reference and repository handoff

- Completed: native/example coverage and required `reference_identity` conformance
  committed as `064e305`, following compiler behavior `71a7baf`. Repository tooling
  is `6f6a0c1`; schema identity corrections are `122b022`. This handoff is the final
  focused documentation/rules commit, including the tooling cache ignore.
- Validation: all 12 combined checks passed; 90 Rust and 18 Python tests pass;
  catalog execution is 9 passed, 14 unsupported, 0 failed in both profiles.
  The release compiler example printed `true`, `false`, `42`, `meowy` exactly.
  Final documentation link and diff whitespace checks passed.
- Blockers: no incomplete implementation edits, active workers or failing checks.
  Full language/library/runtime and distribution gaps remain recorded below.
- Next steps: add guarded result-origin analysis and CFG loan liveness, alongside
  native unwind/cleanup prototyping; then build manifest/module/library integration.

### 2026-09-06 — Commit shared-reference compiler behavior

- Completed: `71a7baf` adds typed storage addresses, pointer lowering, frontend
  capability checks and lexical borrow-origin validation with unit coverage.
- Validation: staged whitespace checks passed; 90 Rust tests, conformance and
  release execution were verified before committing.
- Next steps: commit native reference cases/example and required conformance
  coverage, then finish the cross-project handoff and remaining-work priorities.

### 2026-09-06 — Verify release output and finish review

- Completed: the optimized release compiler built successfully and ran the
  references example with exact stdout `true`, `false`, `42`, `meowy`.
- Validation: final diff whitespace checks pass; independent review found no
  unresolved storage/provenance defect. All 12 combined checks already passed.
- Handoff: tooling worker confirmed ownership of the remaining cache-ignore file;
  it will be included with repository working rules and documentation.
- Next steps: commit compiler implementation, native/example coverage, then the
  final trackers/rules with concrete guarded-borrow and outside-compiler work queued.

### 2026-09-06 — Pass the combined compiler and repository gate

- Completed: all 12 checks in `python3 -B tools/verify.py --all` passed, using the
  pinned host target and fresh compiler binary. No implementation worker remains.
- Validation: 64 library + 26 native tests; 4 compiler harness + 14 tooling tests;
  formatting, Clippy, compiler build, 818 local links, schema integrity, catalog,
  Vim and Neovim. Conformance: 9 passed, 14 unsupported, 0 failed, both profiles.
- Blockers: none for this milestone. Full borrow flow, owners, modules, runtime
  unwinding, library implementation and release qualification remain open.
- Next steps: build/run the release compiler's references example, inspect the
  final diff and commit compiler behavior, source coverage and handoff separately.

### 2026-09-06 — Pass reference identity conformance

- Completed: unchanged `reference_identity` passes in debug/release and is now
  required by the bootstrap harness. Added a runnable reference example and
  documented the narrow storage/lifetime capability boundaries.
- Validation: catalog execution is 9 passed, 14 unsupported, 0 failed. Independent
  native probes passed for null/union addresses, optional fields, recursive local
  borrows and iteration-local references. Full combined gate remains next.
- Decision: E303 is limited to final direct emissions with proven completing
  local result storage; guarded/discarded and other uncertain transfers use B001.
- Next steps: finish new origin regressions, format and run `tools/verify.py --all`,
  then split compiler behavior, source coverage and completed handoff commits.

### 2026-09-06 — Commit repository verification tooling

- Completed: `6f6a0c1` adds repository verification, local-link checking, 14 tests
  and usage documentation. Compiler execution uses an explicit target/output path.
- Validation: all six repository/editor checks and staged whitespace checks pass.
- Next steps: finish discarded-emission diagnostic coverage, document the bounded
  reference support, run the combined gate and commit the compiler feature/tests.

### 2026-09-06 — Commit verified schema identity corrections

- Completed: `122b022` corrects the three distribution references in schema examples.
- Validation: existing schema integrity/rejection checks and staged whitespace
  checks passed. Five focused shared-reference checker/backend/native tests pass.
- Next steps: commit repository tooling independently, finish conservative origin
  diagnostics, and run the combined compiler/repository gate before compiler commits.

### 2026-09-06 — Review discarded reference emissions

- Completed: backend identity/dereference test passes in both profiles. All six
  borrow-origin unit tests and Clippy pass. Fixed a rejection fixture to construct
  its record before union injection so it reaches the intended address boundary.
- Finding: origin validation could report E303 for reference emissions discarded
  by restart/panic or unreachable guards. Emission is not return. The origin-pass
  owner is limiting E303 to proven escapes and using B001 for uncertain transfers.
- Next steps: verify discarded/unreachable emissions, rerun reference checks,
  then run the full combined repository/compiler gate and commit focused slices.

### 2026-09-06 — Integrate origin checks and run first native cases

- Completed: borrow.rs now validates lexical storage, aliases, branches and scoped
  control transfers before backend lowering. Shared identity/dereference tests
  passed in the checker, backend and native source suite.
- Failure: one rejection regression used invalid mutable syntax (`!:`); the
  language uses `:=`. Corrected that test input; no implementation rule changed.
- Validation: three focused tests passed; the scope rejection case needs rerunning.
  Repository tooling now has 14 passing tests and pins its compiler build/run
  target directory, avoiding stale binaries under custom Cargo environments.
- Next steps: rerun reference and origin cases, inspect control-flow handling,
  run full checks and commit the independently verified schema/tooling changes.

### 2026-09-06 — Add shared-reference behavior coverage

- Completed: added source/native cases for copied references, distinct addresses,
  nested record field storage, dereference, local function use and shadowing.
  Rejection coverage includes local escapes and unsupported ownership boundaries.
- Validation: HIR/backend diff checks passed; Cargo verification awaits borrow.rs.
  Baseline Cargo process completed successfully, including doc tests.
- Outside compiler: all six repository/editor checks now pass after correcting
  three stale distribution digests. This is static/editor evidence; the new
  compiler milestone is not yet validated.
- Next steps: complete origin-pass integration, format and run focused reference
  checks, then the full Cargo and repository verification command.

### 2026-09-06 — Implement the bounded reference frontend

- Completed: added shared-reference type handling, physical local/record address
  resolution, scalar/record dereference and immutable reference aliases. Mutable,
  temporary, emitted and parameter storage remain explicit capability boundaries.
- Integration: backend/HIR worker added typed pointer operations; origin-pass
  worker is checking lexical ownership and escaped emissions. Compilation is
  pending the completed pass. Root owns frontend and native tests.
- Outside compiler: 13 new tooling tests, 816 local links, 23 catalog records and
  both editor checks pass. Existing schema examples contain a stale distribution
  digest; tooling worker owns the three affected examples and is correcting it.
- Next steps: integrate origin validation, exercise references in both profiles,
  verify E303 and unsupported cases, then run the full compiler/repository gates.

### 2026-09-06 — Bound shared-reference storage before implementation

- Completed: traced locals, result-slot copies, dispatch receivers and union
  projections. Independent review confirmed proof paths are not physical places.
- Decision: first support immutable ordinary local storage and its record fields,
  shared-reference copies, identity and dereference. Track borrow origins before
  lowering. Mutable roots, exclusive loans, temporary owners and interprocedural
  borrow contracts remain B001. Named emitted bindings cannot yet be borrowed.
- Validation: baseline 53 library and 24 native tests passed. Final Cargo process
  completion still needs polling. Reference fixtures remain unchanged.
- Outside compiler: a worker is implementing repository verification using the
  existing catalog/schema/editor checks plus a tested local-link checker.
- Next steps: record the storage contract in `OWNERSHIP.md`, implement typed
  addresses and origin checks, and test escaped locals and both native profiles.

### 2026-09-06 — Resume after split commits and widen project work

- Completed: confirmed prior work is already split into six focused commits and
  the working tree is clean. Read compiler rules, setup and ownership contract.
- Scope: the user also authorized implementation outside `compiler/`; repository
  verification tooling will progress alongside storage/borrow foundations.
- Validation: no new code yet. Fixture paths are `../docs/conformance/sources/`,
  not `cases/`; correct that path in further investigation.
- Next steps: inspect storage lowering and reference fixtures, record a sound
  bounded implementation design, and delegate independent repository tooling.

### 2026-09-06 — Complete the nullable-value handoff

- Completed: implementation committed as `8209163`; integration coverage and
  example committed as `177169e`. Updated supported behavior, test counts, limits
  and the ordered ownership/borrow next steps. This documentation is the final split.
- Validation: 77 Rust tests and 4 harness tests pass; conformance is 8 passed,
  15 unsupported, 0 failed. Clippy, formatting, release build and example execution pass.
- Blockers: none for starting ownership design; full loop fixed points, resource
  qualification and all other release gaps remain explicitly listed below.
- Next steps: trace reference-identity and temporary-owner fixtures through HIR,
  specify storage/borrow invariants, and record that design before implementation.

### 2026-09-06 — Commit nullable-union compiler behavior

- Completed: committed normalized union HIR, inline LLVM storage/conversions,
  guard analysis, branch narrowing, emission inference and their unit tests.
- Validation: staged whitespace check passed; full runtime/checking evidence is
  recorded in the preceding checkpoints.
- Blockers: none.
- Next steps: commit the native source regressions, nullable example and required
  conformance case, then finalize the ownership-oriented continuation plan.

### 2026-09-06 — Verify the release example and prepare feature commits

- Completed: the release compiler ran `examples/nullable.mwy` and printed exactly
  `meowy`, `null`, `meowy`, `guest`, each on its own line.
- Validation: final tracked diff passed whitespace checks. Changes are limited to
  union/checker/guard code, its tests/example, and compiler documentation.
- Blockers: none.
- Next steps: commit the compiler feature with its unit tests, then native/conformance
  coverage, and finally the completed handoff with ownership work queued next.

### 2026-09-06 — Pass the final union validation gate

- Completed: final combined checks passed: 53 library tests, 24 native tests,
  4 harness tests, Clippy, formatting and the release compiler build.
- Conformance: 8 passed, 15 explicitly unsupported, 0 failed in debug/release;
  `conditional_field` is required. No reference fixtures were changed.
- Blockers: none for this milestone; loop fixed points and ownership remain future work.
- Next steps: smoke-run the release compiler's nullable example, inspect the final
  diff, and commit compiler behavior, integration coverage and updated handoff separately.

### 2026-09-06 — Finish semantic checks and refresh supported behavior

- Completed: all 53 library and 24 native tests passed; independent review found
  no further union correctness issue after the field-key fix. Updated README with
  union support and conservative literal/restart boundaries.
- Validation: Clippy passed. Formatting initially flagged two recently added
  checker assertions; the owner formatted them. The final combined gate is rerunning.
- Limits: a separate 65,536-parameter stress case timed out after 60 seconds;
  general frontend resource/performance qualification remains open.
- Next steps: finish the combined gate, build the release compiler and run the
  nullable example, then commit implementation, integration coverage and handoff separately.

### 2026-09-06 — Close union integration defects

- Completed: field proofs now include storage-type identity and retain original
  tags through narrowed subsets. Discarded-only emission slots preserve operand
  effects without forcing stores into a different completed result type.
- Validation: 13 backend tests and the native restart regression pass in both
  profiles. Guard-budget stress produces B001 rather than a compiler fault.
- Blockers: no confirmed unresolved defect; final source regression checks are next.
- Next steps: run all Rust/native/harness/conformance checks, refresh supported
  behavior and remaining-work documentation, then split the finished feature commits.

### 2026-09-06 — Pass conditional-field conformance and resolve composition edges

- Completed: `conditional_field` now passes both checking profiles; the catalog is
  8 passed, 15 unsupported, 0 failed. It is now a required bootstrap case.
- Validation: 18 checker tests pass. The restart-default native case also passes
  after preserving contextual record fields and skipping stores for discarded-only
  emissions; their operand effects still execute.
- Finding: review found field-refinement keys colliding across different record
  variants sharing a field name. Checker owner is correcting variant identity.
- Next steps: cover same-named fields in distinct union variants, rerun the full
  native suite, then complete Clippy/format/conformance validation and feature commits.

### 2026-09-06 — Exercise nullable source programs

- Completed: ran the complete native source suite with the union implementation.
- Validation: 21 of 22 tests passed, including normalization/retagging, optional
  values, equality, narrowing, mutation invalidation and compressed matcher flow.
- Failure: primary record composition passes only the outer primary expectation
  into a nested block, wrongly rejecting its named field with E207. Checker owner
  is fixing this without manufacturing missing inner fields or bypassing E204/E205.
- Next steps: verify partial record construction and restart defaults, review guard
  soundness, then run the full compiler/conformance gates.

### 2026-09-06 — Verify HIR and native union lowering independently

- Completed: normalized HIR set regression passed independently; backend tests now
  cover scalar/record union payloads, tag conversion, equality, defaults, restart
  clearing and exactly-once evaluation. Guard module formatting/Clippy checks passed.
- Validation: all 11 current backend tests passed, including native union execution
  in debug and release. Full checker semantics and end-to-end regressions remain pending.
- Blockers: none; whole-crate compilation is available again after checker integration.
- Next steps: run the new native source suite and fix any remaining checker/backend
  mismatches before expanding the required conformance set.

### 2026-09-06 — Add native union coverage and validate guard primitives

- Completed: added `examples/nullable.mwy` and native cases for nullable defaults,
  union retagging, equality/formatting, local/field narrowing, short circuiting,
  complementary emissions, mutation invalidation and restart defaults.
- Validation: the standalone guard engine passed five tests, including truth tables,
  correlation, 16,384 independent variables and resource exhaustion. New native tests
  are not yet run; checker integration is still incomplete.
- Blockers: whole-crate compilation temporarily fails while the checker replaces
  old path references. Backend union operations are implemented and adding focused tests.
- Next steps: finish checker integration, run the new native suite, fix concrete
  failures, and promote `conditional_field` to required conformance only after it passes.

### 2026-09-06 — Assign independent implementation work

- Completed: assigned `src/backend.rs` to union lowering, `src/check.rs` to guarded
  inference/narrowing, and `src/flow.rs` to a canonical boolean-guard engine.
  The primary writer owns `src/hir.rs`, integration tests, documentation and STATUS.
- Decision: guard operations expose a bounded-resource failure to the checker;
  union storage recursively writes active fields into zeroed memory to avoid
  reading inactive/padding bytes. No new dependencies or public syntax are needed.
- Validation: design checked against union, equality, emission and narrowing rules;
  compiling the whole feature remains pending integration.
- Next steps: add native cases for optional fields/primaries, union tag remapping,
  active-variant comparisons/printing, flow proofs and their invalidation.

### 2026-09-06 — Add the shared union representation

- Completed: baseline split is complete (`54080b1`, `c82354f`, `f2feea2`). Added
  normalized union-set helpers and explicit coercion/type-test nodes in `src/hir.rs`.
- Validation: helper regression added; full compilation is temporarily incomplete
  until backend and checker handle the new variants. Do not claim this intermediate tree passes.
- Blockers: integration work is expected, not a missing input.
- Next steps: backend owner implements tagged storage; checker owner integrates
  union inference/narrowing with a separately implemented boolean-guard engine.

### 2026-09-06 — Commit integration tests and prepare the feature split

- Completed: integration tests/examples committed as `c82354f`; compiler baseline
  is `54080b1`. Documentation and tracking rules are the third focused commit.
- Validation: both staged diffs passed whitespace checks; working code is unchanged.
- Blockers: none. Read-only design reviews are complete.
- Next steps: add normalized union HIR/coercions/tests, implement LLVM storage and
  guarded checker flow independently, then add native nullable-field regressions.

### 2026-09-06 — Commit the compiler implementation

- Completed: committed the verified Rust frontend/checker/driver, LLVM bridge,
  runtime, toolchain pins and inline unit tests as the first focused commit.
- Validation: staged diff passed whitespace checks; baseline validation is recorded below.
- Blockers: Git writes require the approved escalation because `.git` is read-only
  inside the sandbox. Source edits remain within the workspace.
- Next steps: commit integration tests/examples, then documentation and this tracker;
  start union changes only after these baseline commits are complete.

### 2026-09-06 — Verify the baseline and select guarded union lowering

- Completed: baseline passed 33 library tests, 12 native tests, 4 harness tests,
  Clippy, formatting, and 7 conformance cases (16 explicitly unsupported).
- Decision: use normalized tagged unions with inline maximum-size payload storage,
  explicit coercion/type-test HIR, and shared boolean guards for emission reachability.
  Keep physical local types separate from types proven inside matcher branches.
- Blockers: restart back-edges need conservative handling until full loop dataflow;
  do not accept possible repeated emissions into surviving outer blocks.
- Next steps: commit the verified baseline in three focused pieces, then implement
  HIR/backend union support and guarded checking with native regression cases.

### 2026-09-06 — Establish the next implementation and commit boundaries

- Completed: confirmed conditional slots must join emitted types and add null on
  missing paths; unions require a discriminant and payload storage. Existing HIR
  has neither unions nor dynamic type tests. Started separate checker/backend reviews.
- Validation: inspected the full untracked compiler tree and existing commit style.
- Blockers: none. Shared HIR details are being reviewed before concurrent edits.
- Next steps: refresh baseline checks and commit the existing compiler, integration
  tests/examples, and documentation separately; then implement the reviewed union design.

### 2026-09-06 — Resume nullable-value work

- Completed: reread `AGENTS.md`, the tracker, and current Git status. Existing
  compiler work remains uncommitted and will be preserved.
- Validation: prior results are recorded below; no tests rerun in this step.
- Blockers: none. The union representation and flow-analysis changes need review.
- Next steps: trace the reference fixture through checking and LLVM lowering,
  agree on shared representation, and refresh the baseline tests before editing.

### 2026-09-06 — Verify tracking changes

- Completed: checked both changed documentation files for whitespace errors and
  verified that this file contains `Step log` and `Next steps` sections.
- Validation: all documentation checks passed; no compiler tests rerun for this
  documentation-only change. This resolves the pending check in the previous entry.
- Blockers: none for the next investigation.
- Next steps: follow item 1 below to review nullable emission rules and trace
  `conditional_field`; then record findings and update the implementation steps.

### 2026-09-06 — Require a handoff after every step

- Completed: inspected the existing tracker and rules; updated `AGENTS.md` to
  require a checkpoint and current next steps after every compiler work step.
- Validation: documentation inspected; final diff check pending. Compiler behavior
  is unchanged, so previous runtime results remain the latest execution evidence.
- Blockers: none for the next investigation; full-language gaps remain listed below.
- Next steps: review nullable-field rules and trace `conditional_field` through
  `src/check.rs`, `src/hir.rs`, and `src/backend.rs`; record the proposed representation
  and verification cases before implementing it.

## First milestone checklist

- [x] Inspect reference, conformance catalog, and available toolchain.
- [x] Create an isolated implementation directory and pin the Rust version.
- [x] Establish syntax, typed representation, and diagnostic interfaces.
- [x] Add `AGENTS.md` with project rules and persistent handoff requirements.
- [x] Lossless tokens, original byte offsets, malformed-input recovery.
- [x] Contextual punctuation parser, interpolation, compact/spaced equivalence.
- [x] Lexical names, distinct type namespace, aliases of resolved intrinsics.
- [x] Scalar typing, expected literals, numeric domains and overflow diagnostics.
- [x] Static block fields, emission initialization, emission without return.
- [x] Functions, recursion, forward groups, dispatch, named scope control.
- [x] LLVM verification, object emission, native linking, streaming output.
- [x] Checked arithmetic and short circuiting in debug and release.
- [x] CLI check/build/run, byte-span diagnostics, protected/atomic output paths.
- [x] Native integration tests, selected conformance, explicit unsupported cases.
- [x] README with exact commands and honest supported-feature boundaries.

## Completed union milestone

- [x] Review reference contracts and commit the existing bootstrap separately.
- [x] Add normalized union HIR and explicit coercion/type-test operations.
- [x] Implement and independently test shared boolean guards.
- [x] Integrate union typing, reachable slot inference and branch narrowing.
- [x] Verify tagged LLVM storage, conversions, equality, formatting and defaults.
- [x] Pass new native cases and `conditional_field` in debug/release.
- [x] Refresh documentation, run required checks and make focused feature commits.

## Completed shared-reference milestone

- [x] Record physical storage and lexical lifetime boundaries before implementation.
- [x] Add local/record-field places, pointer identity and copyable dereference.
- [x] Track immutable reference aliases and reject proven final local escapes.
- [x] Keep guarded/discarded transfers and unavailable ownership rules explicit.
- [x] Pass reference identity and native source checks in debug/release.
- [x] Pass the combined repository gate and release compiler example.

## Completed guarded-result and cleanup-protocol milestone

- [x] Reuse frontend guard proofs with unique emission IDs and block completions.
- [x] Propagate all possible bare-reference origins through blocks and aliases.
- [x] Verify complementary selection, named exits and discarded local emissions.
- [x] Prototype explicit native cleanup outside the compiler with bounded storage.
- [x] Verify initialized-only/reverse cleanup, transfer and fatal P008 evidence.
- [x] Include native debug/release/sanitizer checks in repository verification.

Unfinished milestones remain in the current status tracker.
