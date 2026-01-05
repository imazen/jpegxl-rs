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

use half::f16;
use image::DynamicImage;
use pretty_assertions::assert_eq;
use testresult::TestResult;

use crate::decode::Data;
use crate::{
    decoder_builder,
    encode::{ColorEncoding, EncoderFrame, EncoderResult, Metadata},
    encoder_builder, Endianness,
};
use crate::{encode::EncoderSpeed, ResizableRunner, ThreadsRunner};
use std::thread;

fn get_sample() -> DynamicImage {
    image::load_from_memory_with_format(super::SAMPLE_PNG, image::ImageFormat::Png)
        .expect("Failed to get sample file")
}

#[test]
fn simple() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;

    let result: EncoderResult<u16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let decoder = decoder_builder().build().expect("Failed to build decoder");
    let _res = decoder.decode(&result)?;

    Ok(())
}

#[test]
fn jpeg() -> TestResult {
    let threads_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .parallel_runner(&threads_runner)
        .use_container(true)
        .uses_original_profile(true)
        .build()?;

    let res = encoder.encode_jpeg(super::SAMPLE_JPEG)?;

    let (_, Data::Jpeg(reconstructed)) = decoder_builder().build()?.reconstruct(&res)? else {
        panic!("Failed to reconstruct JPEG");
    };

    assert_eq!(super::SAMPLE_JPEG, reconstructed);

    Ok(())
}

#[test]
fn metadata() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;
    encoder.add_metadata(&Metadata::Exif(super::SAMPLE_EXIF), true)?;
    encoder.add_metadata(&Metadata::Xmp(super::SAMPLE_XMP), true)?;

    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn builder() -> TestResult {
    use crate::decode::Metadata;

    let sample = get_sample().to_rgba8();
    let threads_runner = ThreadsRunner::default();

    let mut encoder = encoder_builder()
        .has_alpha(true)
        .lossless(false)
        .speed(EncoderSpeed::Lightning)
        .quality(3.0)
        .color_encoding(ColorEncoding::LinearSrgb)
        .decoding_speed(4)
        .init_buffer_size(64)
        .parallel_runner(&threads_runner)
        .build()?;

    let res: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(4),
        sample.width(),
        sample.height(),
    )?;

    let decoder = decoder_builder().build().unwrap();
    let (
        Metadata {
            num_color_channels,
            has_alpha_channel,
            ..
        },
        _,
    ) = decoder.decode(&res)?;
    assert_eq!(num_color_channels, 3);
    assert!(has_alpha_channel);

    Ok(())
}

#[test]
fn resizable() -> TestResult {
    let resizable_runner = ResizableRunner::default();
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder()
        .parallel_runner(&resizable_runner)
        .build()?;

    let _res: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn pixel_type() -> TestResult {
    let mut encoder = encoder_builder().has_alpha(true).build()?;
    let decoder = decoder_builder().build()?;
    let sample = get_sample().to_rgba8();

    // Check different pixel format
    let frame = EncoderFrame::new(sample.as_raw()).num_channels(4);
    let _res: EncoderResult<u16> = encoder.encode_frame(&frame, sample.width(), sample.height())?;
    let _res: EncoderResult<f16> = encoder.encode_frame(&frame, sample.width(), sample.height())?;
    let _res: EncoderResult<f32> = encoder.encode_frame(&frame, sample.width(), sample.height())?;

    encoder.has_alpha = false;
    let sample = get_sample().to_rgb8();
    let _: EncoderResult<u16> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let res: EncoderResult<f16> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    decoder.decode(&res)?;
    let _: EncoderResult<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    let sample = get_sample().to_rgb32f();
    let _: EncoderResult<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    Ok(())
}

#[test]
fn multi_frames() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().use_container(true).build()?;

    let frame = EncoderFrame::new(sample.as_raw())
        .endianness(Endianness::Native)
        .align(0);

    let result: EncoderResult<f32> = encoder
        .multiple(sample.width(), sample.height())?
        .add_frame(&frame)?
        .add_frame(&frame)?
        .encode()?;
    let decoder = decoder_builder().build()?;
    let _res = decoder.decode(&result)?;

    encoder.uses_original_profile = true;
    let result: EncoderResult<f32> = encoder
        .multiple(sample.width(), sample.height())?
        .add_jpeg_frame(super::SAMPLE_JPEG)?
        .add_jpeg_frame(super::SAMPLE_JPEG)?
        .encode()?;
    let _res = decoder.reconstruct(&result)?;

    Ok(())
}

#[test]
fn gray() -> TestResult {
    let sample = get_sample().to_luma8();
    let mut encoder = encoder_builder()
        .color_encoding(ColorEncoding::SrgbLuma)
        .build()?;
    let decoder = decoder_builder().build()?;

    let result: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;
    _ = decoder.decode(&result)?;

    encoder.color_encoding = Some(ColorEncoding::LinearSrgbLuma);
    let result: EncoderResult<u8> = encoder.encode_frame(
        &EncoderFrame::new(sample.as_raw()).num_channels(1),
        sample.width(),
        sample.height(),
    )?;
    _ = decoder.decode(&result)?;

    encoder.set_frame_option(
        jpegxl_sys::encoder::encode::JxlEncoderFrameSettingId::BrotliEffort,
        1,
    )?;

    Ok(())
}

#[test]
fn initial_buffer() -> TestResult {
    let mut encoder = encoder_builder().init_buffer_size(0).build()?;
    let sample = get_sample().to_rgb8();
    let _: EncoderResult<u16> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let _: EncoderResult<f16> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let _: EncoderResult<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    Ok(())
}

#[test]
fn send_encoder_between_threads() -> TestResult {
    let sample = get_sample().to_rgb8();
    let width = sample.width();
    let height = sample.height();
    let data = sample.into_raw();

    let mut encoder = encoder_builder().build()?;

    // Move encoder to another thread and encode there
    let handle = thread::spawn(move || {
        let result: EncoderResult<u8> = encoder.encode(&data, width, height).unwrap();
        result.data.len()
    });

    let len = handle.join().expect("Thread panicked");
    assert!(len > 0);

    Ok(())
}

#[test]
fn encoder_reuse() -> TestResult {
    let sample = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;

    // Encode the same image multiple times
    let result1: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;
    let result2: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    // Results should be identical for the same input
    assert_eq!(
        result1.data.len(),
        result2.data.len(),
        "Encoded sizes should match"
    );
    assert_eq!(result1.data, result2.data, "Encoded data should match");

    Ok(())
}

#[test]
fn all_speed_settings() -> TestResult {
    let sample = get_sample().to_rgb8();
    let decoder = decoder_builder().build()?;

    let speeds = [
        EncoderSpeed::Lightning,
        EncoderSpeed::Thunder,
        EncoderSpeed::Falcon,
        EncoderSpeed::Cheetah,
        EncoderSpeed::Hare,
        EncoderSpeed::Wombat,
        EncoderSpeed::Squirrel,
        EncoderSpeed::Kitten,
        EncoderSpeed::Tortoise,
    ];

    for speed in speeds {
        let mut encoder = encoder_builder().speed(speed).build()?;
        let result: EncoderResult<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        // Verify it decodes correctly
        let (meta, _) = decoder.decode(&result)?;
        assert_eq!(meta.width, sample.width());
        assert_eq!(meta.height, sample.height());
    }

    Ok(())
}

#[test]
fn quality_settings() -> TestResult {
    let sample = get_sample().to_rgb8();
    let decoder = decoder_builder().build()?;

    // Test various quality levels (0.0 = lossless, higher = more lossy)
    for quality in [0.0, 0.5, 1.0, 2.0, 3.0, 4.0] {
        let mut encoder = encoder_builder()
            .quality(quality)
            .speed(EncoderSpeed::Lightning) // Fast for testing
            .build()?;
        let result: EncoderResult<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        // Verify it decodes correctly
        let (meta, _) = decoder.decode(&result)?;
        assert_eq!(meta.width, sample.width());
        assert_eq!(meta.height, sample.height());
    }

    Ok(())
}

#[test]
fn lossless_encoding() -> TestResult {
    let sample = get_sample().to_rgb8();

    // Lossless encoding requires uses_original_profile=true
    let mut encoder = encoder_builder()
        .lossless(true)
        .uses_original_profile(true)
        .build()?;
    let result: EncoderResult<u8> =
        encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

    // Decode and verify pixel-perfect reconstruction
    let decoder = decoder_builder()
        .pixel_format(crate::decode::PixelFormat {
            num_channels: 3,
            ..Default::default()
        })
        .build()?;
    let (meta, pixels) = decoder.decode_with::<u8>(&result)?;

    assert_eq!(meta.width, sample.width());
    assert_eq!(meta.height, sample.height());
    assert_eq!(pixels.len(), sample.as_raw().len());
    assert_eq!(&pixels, sample.as_raw(), "Lossless should be pixel-perfect");

    Ok(())
}

#[test]
fn all_color_encodings() -> TestResult {
    let sample = get_sample().to_rgb8();
    let decoder = decoder_builder().build()?;

    let encodings = [ColorEncoding::Srgb, ColorEncoding::LinearSrgb];

    for encoding in encodings {
        let mut encoder = encoder_builder().color_encoding(encoding).build()?;
        let result: EncoderResult<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        // Verify it decodes correctly
        let (meta, _) = decoder.decode(&result)?;
        assert_eq!(meta.width, sample.width());
        assert_eq!(meta.height, sample.height());
    }

    Ok(())
}

#[test]
fn rgba_encoding() -> TestResult {
    let sample = get_sample().to_rgba8();
    let decoder = decoder_builder().build()?;

    let mut encoder = encoder_builder().has_alpha(true).build()?;
    let frame = EncoderFrame::new(sample.as_raw()).num_channels(4);
    let result: EncoderResult<u8> =
        encoder.encode_frame(&frame, sample.width(), sample.height())?;

    // Verify it decodes correctly with alpha
    let (meta, _) = decoder.decode(&result)?;
    assert_eq!(meta.width, sample.width());
    assert_eq!(meta.height, sample.height());
    assert!(meta.has_alpha_channel);

    Ok(())
}

#[test]
fn encode_different_pixel_types() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Test u8
    let sample_u8 = get_sample().to_rgb8();
    let mut encoder = encoder_builder().build()?;
    let result: EncoderResult<u8> =
        encoder.encode(sample_u8.as_raw(), sample_u8.width(), sample_u8.height())?;
    decoder.decode(&result)?;

    // Test u16
    let sample_u16 = get_sample().to_rgb16();
    let result: EncoderResult<u16> =
        encoder.encode(sample_u16.as_raw(), sample_u16.width(), sample_u16.height())?;
    decoder.decode(&result)?;

    // Test f32
    let sample_float = get_sample().to_rgb32f();
    let result: EncoderResult<f32> = encoder.encode(
        sample_float.as_raw(),
        sample_float.width(),
        sample_float.height(),
    )?;
    decoder.decode(&result)?;

    Ok(())
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn small_images() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Test encoding very small images (sizes are known to fit in u32)
    for size in [1_u32, 2, 4, 8, 16] {
        let data: Vec<u8> = vec![128; (size * size * 3) as usize];
        let mut encoder = encoder_builder().build()?;
        let result: EncoderResult<u8> = encoder.encode(&data, size, size)?;

        let (meta, _) = decoder.decode(&result)?;
        assert_eq!(meta.width, size);
        assert_eq!(meta.height, size);
    }

    Ok(())
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn non_square_images() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Test non-square dimensions (sizes are known to fit in u32)
    let dimensions: [(u32, u32); 5] = [(100, 50), (50, 100), (1, 100), (100, 1), (7, 13)];

    for (width, height) in dimensions {
        let data: Vec<u8> = vec![128; (width * height * 3) as usize];
        let mut encoder = encoder_builder().build()?;
        let result: EncoderResult<u8> = encoder.encode(&data, width, height)?;

        let (meta, _) = decoder.decode(&result)?;
        assert_eq!(meta.width, width);
        assert_eq!(meta.height, height);
    }

    Ok(())
}
