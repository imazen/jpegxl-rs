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

use std::io::Cursor;

use half::f16;
use image::ImageDecoder;
use pretty_assertions::assert_eq;
use testresult::TestResult;

use crate::{
    common::Endianness,
    decode::{Data, Metadata, PixelFormat, Pixels},
    decoder_builder, DecodeError,
};
use crate::{ResizableRunner, ThreadsRunner};
use std::thread;

#[test]
fn invalid() -> TestResult {
    let decoder = decoder_builder().build()?;

    assert!(matches!(
        decoder.decode(&[0x00, 0x00]),
        Err(DecodeError::InvalidInput)
    ));

    Ok(())
}

#[test]
fn simple() -> TestResult {
    let decoder = decoder_builder().icc_profile(true).build()?;

    let (
        Metadata {
            width,
            height,
            icc_profile,
            ..
        },
        data,
    ) = decoder.decode(super::SAMPLE_JXL)?;

    let Pixels::Uint16(data) = data else {
        panic!("Expected Uint16 pixels");
    };

    assert_eq!(data.len(), (width * height * 4) as usize);
    // Check if icc profile is valid
    lcms2::Profile::new_icc(&icc_profile.expect("ICC profile not retrieved"))?;

    Ok(())
}

#[test]
fn sample_2bit() -> TestResult {
    let decoder = decoder_builder().build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode(super::SAMPLE_JXL_2BIT)?;
    let Pixels::Uint8(data) = data else {
        panic!("Expected Uint8 pixels");
    };
    assert_eq!(data.len(), (width * height * 3) as usize);

    Ok(())
}

#[test]
fn sample_gray() -> TestResult {
    let decoder = decoder_builder().build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode(super::SAMPLE_JXL_GRAY)?;
    let Pixels::Uint16(data) = data else {
        panic!("Expected Uint16 pixels");
    };
    assert_eq!(data.len(), (width * height) as usize);

    Ok(())
}

#[test]
fn pixel_types() -> TestResult {
    let mut decoder = decoder_builder().build()?;

    // Check different pixel types
    decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    decoder.decode_with::<u8>(super::SAMPLE_JXL)?;
    decoder.decode_with::<u16>(super::SAMPLE_JXL)?;
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    // Check endianness
    decoder.pixel_format = Some(PixelFormat {
        endianness: Endianness::Big,
        ..Default::default()
    });
    decoder.decode_with::<u16>(super::SAMPLE_JXL)?;
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    decoder.pixel_format = Some(PixelFormat {
        endianness: Endianness::Little,
        ..Default::default()
    });
    decoder.decode_with::<f16>(super::SAMPLE_JXL)?;

    Ok(())
}

#[test]
fn jpeg() -> TestResult {
    let decoder = decoder_builder().init_jpeg_buffer(512).build()?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL_JPEG)?;
    let Data::Jpeg(data) = data else {
        panic!("Expected JPEG reconstruction");
    };

    let jpeg = image::codecs::jpeg::JpegDecoder::new(Cursor::new(data))?;
    let mut v = vec![0; jpeg.total_bytes().try_into().unwrap()];
    jpeg.read_image(&mut v)?;

    let (_, data) = decoder.reconstruct(super::SAMPLE_JXL)?;
    assert!(matches!(data, Data::Pixels(Pixels::Uint16(_))));

    Ok(())
}

#[test]
fn builder() -> TestResult {
    use crate::decode::ProgressiveDetail;

    let threads_runner = ThreadsRunner::default();
    let resizable_runner = ResizableRunner::default();
    let mut decoder = decoder_builder()
        .pixel_format(PixelFormat {
            num_channels: 3,
            endianness: Endianness::Big,
            align: 10,
        })
        .desired_intensity_target(0.5)
        .coalescing(false)
        .progressive_detail(ProgressiveDetail::Passes)
        .render_spotcolors(false)
        .decompress(true)
        .unpremul_alpha(true)
        .skip_reorientation(true)
        .parallel_runner(&resizable_runner)
        .build()?;

    let (Metadata { width, height, .. }, data) = decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (width * height * 3) as usize);

    // Set options after creating decoder
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 4,
        endianness: Endianness::Little,
        ..PixelFormat::default()
    });
    decoder.skip_reorientation = Some(true);
    decoder.parallel_runner = Some(&threads_runner);

    decoder.decode(super::SAMPLE_JXL)?;
    let (Metadata { width, height, .. }, data) = decoder.decode_with::<f32>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (width * height * 4) as usize);

    Ok(())
}

#[test]
fn truncated_data() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Test with various truncation points
    for len in [0, 1, 10, 50, 100, 500] {
        if len < super::SAMPLE_JXL.len() {
            let result = decoder.decode(&super::SAMPLE_JXL[..len]);
            assert!(
                result.is_err(),
                "Expected error for truncated data of length {len}"
            );
        }
    }

    Ok(())
}

#[test]
fn metadata_values() -> TestResult {
    let decoder = decoder_builder().build()?;

    let (metadata, _) = decoder.decode(super::SAMPLE_JXL)?;

    // Verify metadata is reasonable
    assert!(metadata.width > 0, "Width should be positive");
    assert!(metadata.height > 0, "Height should be positive");
    assert!(
        metadata.num_color_channels == 1 || metadata.num_color_channels == 3,
        "Color channels should be 1 or 3"
    );
    assert!(
        metadata.intensity_target > 0.0,
        "Intensity target should be positive"
    );

    Ok(())
}

#[test]
fn decoder_reuse() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Decode the same image multiple times
    for _ in 0..3 {
        let (meta1, pixels1) = decoder.decode(super::SAMPLE_JXL)?;
        let (meta2, pixels2) = decoder.decode(super::SAMPLE_JXL)?;

        assert_eq!(meta1.width, meta2.width);
        assert_eq!(meta1.height, meta2.height);

        // Verify pixel data is consistent
        match (pixels1, pixels2) {
            (Pixels::Uint16(p1), Pixels::Uint16(p2)) => {
                assert_eq!(p1.len(), p2.len());
                assert_eq!(p1, p2, "Pixel data should be identical across decodes");
            }
            _ => panic!("Expected same pixel type"),
        }
    }

    Ok(())
}

#[test]
fn send_decoder_between_threads() -> TestResult {
    let decoder = decoder_builder().build()?;

    // Move decoder to another thread and decode there
    let handle = thread::spawn(move || {
        let (metadata, _) = decoder.decode(super::SAMPLE_JXL).unwrap();
        metadata.width
    });

    let width = handle.join().expect("Thread panicked");
    assert!(width > 0);

    Ok(())
}

#[test]
fn different_channel_counts() -> TestResult {
    // Request 3 channels (RGB)
    let mut decoder = decoder_builder()
        .pixel_format(PixelFormat {
            num_channels: 3,
            ..Default::default()
        })
        .build()?;
    let (meta, data) = decoder.decode_with::<u8>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (meta.width * meta.height * 3) as usize);

    // Request 4 channels (RGBA)
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 4,
        ..Default::default()
    });
    let (meta, data) = decoder.decode_with::<u8>(super::SAMPLE_JXL)?;
    assert_eq!(data.len(), (meta.width * meta.height * 4) as usize);

    // Request 1 channel (grayscale) - only works with grayscale images
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 1,
        ..Default::default()
    });
    let (meta, data) = decoder.decode_with::<u8>(super::SAMPLE_JXL_GRAY)?;
    assert_eq!(data.len(), (meta.width * meta.height) as usize);

    // Request 2 channels (grayscale + alpha)
    decoder.pixel_format = Some(PixelFormat {
        num_channels: 2,
        ..Default::default()
    });
    let (meta, data) = decoder.decode_with::<u8>(super::SAMPLE_JXL_GRAY)?;
    assert_eq!(data.len(), (meta.width * meta.height * 2) as usize);

    Ok(())
}

#[test]
fn alignment_options() -> TestResult {
    // Test various alignment values
    for align in [0, 1, 4, 8, 16, 32] {
        let decoder = decoder_builder()
            .pixel_format(PixelFormat {
                num_channels: 3,
                endianness: Endianness::Native,
                align,
            })
            .build()?;

        let (meta, data) = decoder.decode_with::<u8>(super::SAMPLE_JXL)?;
        assert!(
            data.len() >= (meta.width * meta.height * 3) as usize,
            "Data should have at least width*height*3 bytes for align={align}"
        );
    }

    Ok(())
}

#[test]
fn grayscale_image() -> TestResult {
    let decoder = decoder_builder().build()?;

    let (metadata, pixels) = decoder.decode(super::SAMPLE_JXL_GRAY)?;

    // Grayscale should have 1 color channel
    assert_eq!(metadata.num_color_channels, 1);

    // Verify we got the right pixel type and count
    let Pixels::Uint16(data) = pixels else {
        panic!("Expected Uint16 pixels for grayscale");
    };
    assert_eq!(data.len(), (metadata.width * metadata.height) as usize);

    Ok(())
}

#[test]
fn decode_without_icc() -> TestResult {
    let decoder = decoder_builder().icc_profile(false).build()?;

    let (metadata, _) = decoder.decode(super::SAMPLE_JXL)?;
    assert!(
        metadata.icc_profile.is_none(),
        "ICC profile should not be retrieved when disabled"
    );

    Ok(())
}

#[test]
fn decode_with_icc() -> TestResult {
    let decoder = decoder_builder().icc_profile(true).build()?;

    let (metadata, _) = decoder.decode(super::SAMPLE_JXL)?;
    assert!(
        metadata.icc_profile.is_some(),
        "ICC profile should be retrieved when enabled"
    );

    let icc = metadata.icc_profile.unwrap();
    assert!(!icc.is_empty(), "ICC profile should not be empty");

    Ok(())
}
