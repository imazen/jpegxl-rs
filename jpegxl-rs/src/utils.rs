/*
This file is part of jpegxl-rs.

jpegxl-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-rs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Utils functions when a decoder or encoder is not needed

use jpegxl_sys::decode::{JxlSignature, JxlSignatureCheck};

/// Check if the signature of the input is valid.
/// Return `None` if it needs more data.
#[must_use]
pub fn check_valid_signature(buf: &[u8]) -> Option<bool> {
    use JxlSignature::{Codestream, Container, Invalid, NotEnoughBytes};

    match unsafe { JxlSignatureCheck(buf.as_ptr(), buf.len()) } {
        NotEnoughBytes => None,
        Invalid => Some(false),
        Codestream | Container => Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::SAMPLE_JXL;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_signature() {
        assert!(check_valid_signature(&[]).is_none());
        assert_eq!(check_valid_signature(&[0; 64]), Some(false));
        assert_eq!(check_valid_signature(SAMPLE_JXL), Some(true));
    }

    #[test]
    fn test_signature_partial_data() {
        // Very short data should return None (need more bytes)
        assert!(check_valid_signature(&[0]).is_none());
        assert!(check_valid_signature(&[0, 0]).is_none());

        // JXL codestream signature starts with 0xFF 0x0A
        assert_eq!(check_valid_signature(&[0xFF, 0x0A]), Some(true));

        // JXL container signature (ISOBMFF box)
        let container_sig = [
            0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
        ];
        assert_eq!(check_valid_signature(&container_sig), Some(true));
    }

    #[test]
    fn test_signature_invalid_data() {
        // Clearly invalid signatures
        assert_eq!(
            check_valid_signature(&[0x89, 0x50, 0x4E, 0x47]),
            Some(false)
        ); // PNG
        assert_eq!(
            check_valid_signature(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(false)
        ); // JPEG
        assert_eq!(
            check_valid_signature(&[0x47, 0x49, 0x46, 0x38]),
            Some(false)
        ); // GIF
        assert_eq!(check_valid_signature(b"not a jxl file"), Some(false));
    }
}
