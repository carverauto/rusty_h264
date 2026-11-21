// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A CAVLC slice must end with its `rbsp_slice_trailing_bits`.
//!
//! When a slice's macroblock data ends on a byte boundary, its last byte is
//! 0x80: the `rbsp_stop_one_bit` and seven alignment zeros. If that byte is
//! lost, every macroblock still parses, and the decoder accepted the slice:
//! after the last macroblock it only asked whether the reader had reached the
//! last set bit, and the last set bit was now a macroblock bit it had already
//! consumed. The slice data may not extend past the stop bit (spec §7.3.2.8,
//! §7.3.4); the decoder now reports a slice that did as
//! `DecodeError::Truncated`. ffmpeg flags the picture as corrupt.
//!
//! The vector is x264 0.165 (CLI), Baseline (CAVLC), 96x64, five pictures,
//! `--slice-max-mbs 2`, over a synthetic test pattern (`testsrc2` plus
//! temporal noise, blurred), QP 28, SEI removed (`cavlc_trailing_ok.264`). In
//! `cavlc_trailing_missing.264` the first slice whose RBSP ends in a lone 0x80
//! (the IDR picture's slice at macroblock 12) lost that byte. Before the fix it
//! decoded to five frames with no error.

use rusty_h264_decoder::{split_access_units, DecodeError, Decoder};

#[test]
fn complete_cavlc_slices_decode() {
    let stream = include_bytes!("../../../tests/cavlc_data/cavlc_trailing_ok.264");
    assert_eq!(Decoder::new().decode_stream(stream).map(|f| f.len()), Ok(5));
}

#[test]
fn cavlc_slice_without_trailing_bits_is_an_error() {
    let stream = include_bytes!("../../../tests/cavlc_data/cavlc_trailing_missing.264");
    // The damaged slice belongs to the first picture: the first event is the
    // error, before any picture is produced.
    let mut d = Decoder::new();
    let first = split_access_units(stream)
        .into_iter()
        .map(|unit| d.decode(unit))
        .find(|r| !matches!(r, Ok(None)));
    assert!(
        matches!(first, Some(Err(DecodeError::Truncated))),
        "{:?}",
        first.map(|r| r.map(|f| f.is_some()))
    );
    assert_eq!(
        Decoder::new().decode_stream(stream).err(),
        Some(DecodeError::Truncated)
    );
}
