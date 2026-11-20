//! Image decoding for the standalone Android platform, without a Java VM.

#[unsafe(no_mangle)]
unsafe extern "C" fn mln_rust_decode_image(
    data: *const u8,
    length: usize,
    context: *mut std::ffi::c_void,
    receive: extern "C" fn(*mut std::ffi::c_void, u32, u32, *const u8, usize),
) -> bool {
    if data.is_null() || length == 0 {
        return false;
    }
    // The C++ caller borrows a std::string for this synchronous call.
    let bytes = unsafe { std::slice::from_raw_parts(data, length) };
    match image::load_from_memory(bytes) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            receive(context, rgba.width(), rgba.height(), rgba.as_ptr(), rgba.len());
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::mln_rust_decode_image;
    use std::{ffi::c_void, io::Cursor};

    extern "C" fn receive(
        context: *mut c_void,
        width: u32,
        height: u32,
        bytes: *const u8,
        length: usize,
    ) {
        // The test supplies a live output slot and the decoder owns the RGBA bytes.
        let output = unsafe { &mut *context.cast::<Option<(u32, u32, Vec<u8>)>>() };
        let pixels = unsafe { std::slice::from_raw_parts(bytes, length) };
        *output = Some((width, height, pixels.to_vec()));
    }

    #[test]
    fn image_decoder_preserves_rgba_dimensions_and_alpha() {
        let pixels = vec![255, 0, 0, 128, 0, 128, 255, 255];
        let image = image::RgbaImage::from_raw(2, 1, pixels.clone()).unwrap();
        let mut encoded = Cursor::new(Vec::new());
        image.write_to(&mut encoded, image::ImageFormat::Png).unwrap();
        let encoded = encoded.into_inner();
        let mut output: Option<(u32, u32, Vec<u8>)> = None;
        // The encoded buffer and output slot remain live through the callback.
        let success = unsafe {
            mln_rust_decode_image(
                encoded.as_ptr(),
                encoded.len(),
                std::ptr::from_mut(&mut output).cast(),
                receive,
            )
        };
        assert!(success);
        assert_eq!(output, Some((2, 1, pixels)));
    }

    #[test]
    fn invalid_images_do_not_call_the_receiver() {
        let mut output: Option<(u32, u32, Vec<u8>)> = None;
        for bytes in [&b"not an image"[..], &b""[..]] {
            // Both pointers are valid for the duration of this synchronous call.
            let success = unsafe {
                mln_rust_decode_image(
                    bytes.as_ptr(),
                    bytes.len(),
                    std::ptr::from_mut(&mut output).cast(),
                    receive,
                )
            };
            assert!(!success);
            assert!(output.is_none());
        }
        // The FFI entry point rejects a null buffer before constructing a slice.
        assert!(!unsafe {
            mln_rust_decode_image(
                std::ptr::null(),
                1,
                std::ptr::from_mut(&mut output).cast(),
                receive,
            )
        });
        assert!(output.is_none());
    }
}
