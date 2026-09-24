const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
const IHDR_WIDTH_OFFSET: usize = 16;
const IHDR_HEIGHT_OFFSET: usize = 20;
const MIN_HEADER_LENGTH: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

fn read_u32_be(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Reads the dimensions out of the PNG header.
///
/// The terminal reads the size itself when displaying, but the CLI needs it to
/// decide how many cells the image should occupy.
pub fn read_png_size(png: &[u8]) -> Option<PixelSize> {
    if png.len() < MIN_HEADER_LENGTH || png[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return None;
    }

    let width = read_u32_be(png, IHDR_WIDTH_OFFSET);
    let height = read_u32_be(png, IHDR_HEIGHT_OFFSET);

    if width == 0 || height == 0 {
        return None;
    }

    Some(PixelSize { width, height })
}

#[cfg(test)]
mod tests {
    use super::{read_png_size, PixelSize, PNG_SIGNATURE};

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn reads_the_dimensions() {
        assert_eq!(
            read_png_size(&png_header(128, 64)),
            Some(PixelSize {
                width: 128,
                height: 64
            })
        );
    }

    #[test]
    fn rejects_anything_that_is_not_a_png() {
        assert_eq!(read_png_size(b"this is not a png file at all"), None);
        assert_eq!(read_png_size(&png_header(128, 64)[..10]), None);
        assert_eq!(read_png_size(&png_header(0, 64)), None);
    }
}
