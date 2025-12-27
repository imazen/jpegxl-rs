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

- [x] FFI safety and correctness - Well-structured, proper #[repr(C)] annotations
- [x] Memory management (encoder/decoder lifecycle) - Proper Drop impls, Send implemented
- [x] Error handling completeness - #[non_exhaustive], NotImplemented variant added
- [x] API ergonomics and consistency - Good builder pattern usage
- [ ] Documentation quality - Most public types documented, room for improvement
- [ ] Test coverage - Good basic coverage (29 tests), could add edge cases
- [x] CI/CD pipeline - Comprehensive (multi-platform, sanitizers, coverage)
- [x] Dependency hygiene - Dependabot configured
- [x] MSRV policy and testing - Set to 1.81.0, tested in CI
- [ ] Feature flag organization - image feature well-organized, vendored for static linking

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

### 2024 - Initial Audit

#### Completed Fixes

1. **Clippy warnings** (commit 40f4184):
   - Fixed `assert!(!cfg!(tsan), ...)` constant assertion warning in jpegxl-src
   - Fixed `derivable_impls` warning for EncoderSpeed enum
   - Added `#[non_exhaustive]` to DecodeError and EncodeError enums

2. **Soundness improvements** (commit 10937d0):
   - Replaced `todo!()` panics with `DecodeError::NotImplemented` errors
   - Combined informational decoder events (FullImage, Frame, FrameProgression)
   - Added `unsafe impl Send` for JxlDecoder and JxlEncoder
   - Documented thread safety: decoder/encoder can be sent between threads but are NOT Sync

#### Architecture Notes

- **Three crates**: jpegxl-rs (high-level), jpegxl-sys (FFI), jpegxl-src (build from source)
- **FFI bindings** are well-structured with proper `#[repr(C)]` annotations
- **Memory management**: Encoder/decoder use raw pointers with proper Drop impls
- **Error types** have `#[non_exhaustive]` for API stability

#### CI/CD Status

The CI pipeline is comprehensive:
- Multi-platform tests (Ubuntu, macOS Intel/ARM, Windows)
- Code coverage via llvm-cov and codecov
- AddressSanitizer and ThreadSanitizer testing
- MSRV checking with cargo-hack
- Clippy with -D warnings
- Rustfmt checking

#### Test Coverage Expansion

Test count increased from 29 to 53 tests:

Decoder tests:
- truncated_data, metadata_values, decoder_reuse
- send_decoder_between_threads, different_channel_counts
- alignment_options, grayscale_image, decode_without_icc, decode_with_icc

Encoder tests:
- send_encoder_between_threads, encoder_reuse
- all_speed_settings, quality_settings, lossless_encoding
- all_color_encodings, rgba_encoding, encode_different_pixel_types
- small_images, non_square_images

Utils tests:
- test_signature_partial_data, test_signature_invalid_data

Image integration tests:
- grayscale_to_image, image_dimensions, rgba_to_image

#### Areas Still Needing Work

- [ ] Consider adding benchmarks for new test scenarios
- [ ] Verify all public APIs have docs
