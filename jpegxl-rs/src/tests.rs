mod decode;
mod encode;

pub const SAMPLE_PNG: &[u8] = include_bytes!("../../samples/sample.png");

/// Compile-time assertion that JxlDecoder is Send
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<crate::decode::JxlDecoder>();
};

/// Compile-time assertion that JxlEncoder is Send
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<crate::encode::JxlEncoder>();
};
const SAMPLE_JPEG: &[u8] = include_bytes!("../../samples/sample.jpg");
const SAMPLE_EXIF: &[u8] = include_bytes!("../../samples/sample.exif");
const SAMPLE_XMP: &[u8] = include_bytes!("../../samples/sample.xmp");
pub const SAMPLE_JXL: &[u8] = include_bytes!("../../samples/sample.jxl");
const SAMPLE_JXL_JPEG: &[u8] = include_bytes!("../../samples/sample_jpg.jxl");
pub const SAMPLE_JXL_GRAY: &[u8] = include_bytes!("../../samples/sample_grey.jxl");
const SAMPLE_JXL_2BIT: &[u8] = include_bytes!("../../samples/2bit.jxl");
