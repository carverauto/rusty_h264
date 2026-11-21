// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A CABAC slice cut off mid-data is an error.
//!
//! The arithmetic decoder zero-fills past the end of its buffer, and zeros
//! decode as plausible bins, so a truncated CABAC slice used to run on to its
//! picture's last macroblock and return a full-size picture with a wrong tail
//! and no error (a truncated CAVLC slice was already reported, because its
//! bit reader fails at the end of the data). A complete CABAC slice never
//! reads past its `rbsp_stop_one_bit`; the decoder now reports a slice whose
//! engine did as `DecodeError::Truncated`.
//!
//! The vectors are one x264 stream via ffmpeg 9.0, 64x64, six frames, IPPPPP,
//! High, CABAC, one slice per picture, SEI removed (`cabac_untruncated.264`):
//! `ffmpeg -f lavfi -i testsrc2=s=64x64:r=24:d=0.25 -c:v libx264 -profile:v
//! high -qp 30 -g 48 -bf 0 -x264-params weightp=0 -bsf:v h264_mp4toannexb -f
//! h264`. `cabac_truncated_idr.264` keeps the first 70% of the IDR slice's
//! RBSP, `cabac_truncated_p.264` the first 80% of the third P slice's; ffmpeg
//! reports "error while decoding MB" on both. Before the fix both decoded to
//! six frames with no error.

use rusty_h264_decoder::{split_access_units, DecodeError, Decoder};

/// Feeds `stream` to [`Decoder::decode`] unit by unit; returns `Ok(picture)`
/// / `Err(..)` per unit that produced either, in order.
fn events(stream: &[u8]) -> Vec<Result<(), DecodeError>> {
    let mut d = Decoder::new();
    split_access_units(stream)
        .into_iter()
        .filter_map(|unit| match d.decode(unit) {
            Ok(Some(_)) => Some(Ok(())),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect()
}

#[test]
fn complete_cabac_stream_decodes() {
    let stream = include_bytes!("../../../tests/cabac_data/cabac_untruncated.264");
    assert_eq!(events(stream), vec![Ok(()); 6]);
}

#[test]
fn truncated_cabac_idr_slice_is_an_error() {
    let stream = include_bytes!("../../../tests/cabac_data/cabac_truncated_idr.264");
    assert_eq!(events(stream).first(), Some(&Err(DecodeError::Truncated)));
    assert_eq!(
        Decoder::new().decode_stream(stream).err(),
        Some(DecodeError::Truncated)
    );
}

#[test]
fn truncated_cabac_p_slice_is_an_error() {
    let stream = include_bytes!("../../../tests/cabac_data/cabac_truncated_p.264");
    // Pictures 0..=2 are intact; the fourth access unit is the cut one.
    let got = events(stream);
    assert_eq!(got[..3], [Ok(()), Ok(()), Ok(())]);
    assert_eq!(got.get(3), Some(&Err(DecodeError::Truncated)));
}
