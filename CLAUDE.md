# jpegxl-rs-bindings Development Instructions

## Prime Directive

**Keep working. Dig deeper. Fix everything. Do not stop.**

When you find an issue, fix it. When you fix something, look for related issues. When tests pass, look for edge cases. When code works, make it better. Production-ready means:
- Every public API is sound and well-documented
- Every error case is handled properly
- Every unsafe block is justified and correct
- Every test actually tests something meaningful
- No warnings from rustc or clippy
- No TODO/FIXME left unaddressed

## Workflow

1. **Explore exhaustively** - Read all source files, understand the full picture
2. **Fix incrementally** - Commit each logical fix separately
3. **Test continuously** - Run `cargo fmt && cargo clippy -- -D warnings && cargo test` after each change
4. **Document findings** - Update this file with discoveries and decisions

## Quality Standards

- All public types need documentation
- All unsafe code needs safety comments
- All error types need `#[non_exhaustive]`
- All APIs need to be consistent and ergonomic
- Integration tests with real JPEG XL files
- Benchmarks for performance-critical paths

## Known Areas to Investigate

- [ ] FFI safety and correctness
- [ ] Memory management (encoder/decoder lifecycle)
- [ ] Error handling completeness
- [ ] API ergonomics and consistency
- [ ] Documentation quality
- [ ] Test coverage
- [ ] CI/CD pipeline
- [ ] Dependency hygiene
- [ ] MSRV policy and testing
- [ ] Feature flag organization

## Build Commands

```bash
# Full check
cargo fmt && cargo clippy -- -D warnings && cargo test

# With all features
cargo test --all-features

# Build libjxl from source
cargo build --features vendored
```

## Findings Log

(Document discoveries, decisions, and fixes here as you work)
