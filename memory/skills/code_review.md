# Skillset: Code Review & Quality Assurance

## Objective
Provide rigorous, zero-compromise code review for Rust, Python, TypeScript, and Docker infrastructure changes.

## Review Pillars
1. **Safety & Correctness:**
   - Unhandled unwraps or panics in production paths.
   - Resource leaks (unclosed sockets, unbounded queues, memory exhaustion).
   - Concurrency pitfalls (deadlocks, lock contention, race conditions).
2. **Performance & Overhead:**
   - Unnecessary allocations and clones in hot loops.
   - Algorithmic complexity ($O(N^2)$ checks).
   - Inefficient async Tokio blocking calls (`std::thread::sleep` instead of `tokio::time::sleep`).
3. **Architecture & Clean Interfaces:**
   - Clear separation of concerns, explicit error enums.
   - Proper trait abstractions without over-engineering.
4. **Actionable Suggestions:**
   - Always provide before/after diff snippets.
   - Categorize comments as `[Nit]`, `[Suggestion]`, or `[Blocker]`.
