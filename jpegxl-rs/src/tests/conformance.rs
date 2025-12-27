/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Expanded conformance tests using codec-corpus JXL files.
//!
//! These tests use codec-eval to sparse checkout JXL test files from
//! <https://github.com/imazen/codec-corpus>
//!
//! # Usage
//!
//! ```bash
//! cargo test --features conformance-tests,vendored
//! ```

use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};

use codec_eval::corpus::{SparseCheckout, SparseFilter};

use crate::decoder_builder;
use crate::ThreadsRunner;

const CODEC_CORPUS_URL: &str = "https://github.com/imazen/codec-corpus";

/// Get or initialize the corpus directory with JXL files
fn get_corpus() -> &'static PathBuf {
    static CORPUS_PATH: OnceLock<PathBuf> = OnceLock::new();

    CORPUS_PATH.get_or_init(|| {
        // Use target directory for corpus cache
        let corpus_dir = env::var("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("target"))
            .join("codec-corpus");

        // Initialize or update sparse checkout
        let checkout = if corpus_dir.exists() {
            SparseCheckout::init(&corpus_dir).expect("Failed to init existing corpus checkout")
        } else {
            SparseCheckout::clone_shallow(CODEC_CORPUS_URL, &corpus_dir, 1)
                .expect("Failed to clone codec-corpus")
        };

        // Add JXL directory filter
        checkout
            .add_filter(&SparseFilter::Directory("jxl".into()))
            .expect("Failed to add jxl filter");

        // Checkout the files
        checkout.checkout().expect("Failed to checkout jxl files");

        corpus_dir.join("jxl")
    })
}

/// Collect all JXL files from a directory recursively
fn collect_jxl_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_jxl_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "jxl") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

#[test]
fn decode_all_corpus_files() {
    let corpus_path = get_corpus();
    let jxl_files = collect_jxl_files(corpus_path);

    assert!(
        !jxl_files.is_empty(),
        "No JXL files found in corpus at {}",
        corpus_path.display()
    );

    println!("Found {} JXL files in corpus", jxl_files.len());

    let runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&runner)
        .build()
        .expect("Failed to create decoder");

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(PathBuf, String)> = Vec::new();

    for file_path in &jxl_files {
        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(e) => {
                failures.push((file_path.clone(), format!("Failed to read: {e}")));
                failed += 1;
                continue;
            }
        };

        match decoder.decode(&data) {
            Ok((metadata, _)) => {
                // Basic sanity checks
                if metadata.width == 0 || metadata.height == 0 {
                    failures.push((
                        file_path.clone(),
                        format!("Invalid dimensions: {}x{}", metadata.width, metadata.height),
                    ));
                    failed += 1;
                } else {
                    passed += 1;
                }
            }
            Err(e) => {
                failures.push((file_path.clone(), format!("Decode error: {e}")));
                failed += 1;
            }
        }
    }

    println!(
        "Results: {passed} passed, {failed} failed out of {} total",
        jxl_files.len()
    );

    if !failures.is_empty() {
        println!("\nFailures:");
        for (path, error) in &failures {
            println!("  {}: {}", path.display(), error);
        }
    }

    assert!(
        failed == 0,
        "{failed} out of {} files failed to decode",
        jxl_files.len()
    );
}

#[test]
fn decode_corpus_with_all_pixel_types() {
    let corpus_path = get_corpus();
    let jxl_files = collect_jxl_files(corpus_path);

    assert!(!jxl_files.is_empty(), "No JXL files found in corpus");

    // Test a subset of files with different pixel types
    let test_files: Vec<_> = jxl_files.iter().take(10).collect();

    let runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&runner)
        .build()
        .expect("Failed to create decoder");

    for file_path in test_files {
        let data = fs::read(file_path).expect("Failed to read file");

        // Test u8
        if let Err(e) = decoder.decode_with::<u8>(&data) {
            panic!("Failed to decode {} as u8: {}", file_path.display(), e);
        }

        // Test u16
        if let Err(e) = decoder.decode_with::<u16>(&data) {
            panic!("Failed to decode {} as u16: {}", file_path.display(), e);
        }

        // Test f32
        if let Err(e) = decoder.decode_with::<f32>(&data) {
            panic!("Failed to decode {} as f32: {}", file_path.display(), e);
        }
    }
}

#[test]
fn corpus_metadata_validation() {
    let corpus_path = get_corpus();
    let jxl_files = collect_jxl_files(corpus_path);

    assert!(!jxl_files.is_empty(), "No JXL files found in corpus");

    let runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&runner)
        .icc_profile(true)
        .build()
        .expect("Failed to create decoder");

    for file_path in &jxl_files {
        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if let Ok((metadata, _)) = decoder.decode(&data) {
            // Validate metadata constraints
            assert!(
                metadata.width > 0 && metadata.width <= 1_073_741_824,
                "Invalid width {} in {}",
                metadata.width,
                file_path.display()
            );
            assert!(
                metadata.height > 0 && metadata.height <= 1_073_741_824,
                "Invalid height {} in {}",
                metadata.height,
                file_path.display()
            );
            assert!(
                metadata.num_color_channels == 1 || metadata.num_color_channels == 3,
                "Invalid color channels {} in {}",
                metadata.num_color_channels,
                file_path.display()
            );
            assert!(
                metadata.intensity_target > 0.0,
                "Invalid intensity target {} in {}",
                metadata.intensity_target,
                file_path.display()
            );
        }
    }
}

#[test]
fn corpus_roundtrip_encoding() {
    let corpus_path = get_corpus();
    let jxl_files = collect_jxl_files(corpus_path);

    assert!(!jxl_files.is_empty(), "No JXL files found in corpus");

    // Test roundtrip on a subset of files
    let test_files: Vec<_> = jxl_files.iter().take(5).collect();

    let runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&runner)
        .build()
        .expect("Failed to create decoder");

    let mut encoder = crate::encoder_builder()
        .parallel_runner(&runner)
        .speed(crate::encode::EncoderSpeed::Lightning)
        .build()
        .expect("Failed to create encoder");

    use crate::decode::PixelFormat;
    use crate::encode::EncoderResult;

    for file_path in test_files {
        let data = fs::read(file_path).expect("Failed to read file");

        // Decode original
        let dec = decoder_builder()
            .parallel_runner(&runner)
            .pixel_format(PixelFormat {
                num_channels: 3,
                ..Default::default()
            })
            .build()
            .expect("Failed to create decoder");

        let (orig_meta, orig_pixels) = match dec.decode_with::<u8>(&data) {
            Ok(r) => r,
            Err(_) => continue, // Skip files that can't be decoded as RGB
        };

        // Skip animations and special formats for roundtrip test
        if orig_meta.width > 4096 || orig_meta.height > 4096 {
            continue;
        }

        // Re-encode
        let encoded: EncoderResult<u8> =
            match encoder.encode(&orig_pixels, orig_meta.width, orig_meta.height) {
                Ok(r) => r,
                Err(_) => continue,
            };

        // Decode re-encoded
        let (new_meta, _) = decoder
            .decode(&encoded)
            .expect("Failed to decode re-encoded image");

        assert_eq!(
            orig_meta.width,
            new_meta.width,
            "Width mismatch after roundtrip for {}",
            file_path.display()
        );
        assert_eq!(
            orig_meta.height,
            new_meta.height,
            "Height mismatch after roundtrip for {}",
            file_path.display()
        );
    }
}
