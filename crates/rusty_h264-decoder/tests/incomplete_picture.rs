// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A picture with a lost slice is reported, not returned.
//!
//! When a middle slice of a multi-slice picture is missing, the following
//! slices still carry the picture to its last macroblock, so the decoder used
//! to finish it and return a frame with a band of macroblocks no slice ever
//! decoded, and no error. ffmpeg conceals the band and flags the frame
//! ("corrupt decoded frame"). The decoder now returns
//! `DecodeError::Truncated` from the `decode` call that completes such a
//! picture, the variant it already uses for a picture that never reached its
//! last macroblock; its state advances as for a complete picture, so the
//! following pictures still decode.
//!
//! The vectors are x264 0.165 (CLI), 96x64 (6x4 macroblocks), five pictures,
//! `--slice-max-mbs 6` (four slices per picture, one per macroblock row), over
//! a synthetic test pattern (`testsrc2` plus temporal noise, blurred), QP 28,
//! SEI removed. `*_full.264` is the stream as encoded; in the other one slice
//! NAL was deleted:
//! - CABAC, Main, IPPPP: the slice at macroblock 6 of picture 2;
//! - CAVLC, Baseline, IPPPP: the slice at macroblock 12 of the IDR picture.
//!
//! Before the fix both streams decoded to five frames with no error.

use rusty_h264_decoder::{split_access_units, DecodeError, Decoder};

/// Feeds `stream` to [`Decoder::decode`] unit by unit (one slice NAL each, as
/// `split_access_units` cuts it); returns the pictures produced and the errors
/// reported, as `Ok(picture count so far)` / `Err(..)` events in order.
fn events(stream: &[u8]) -> Vec<Result<usize, DecodeError>> {
    let mut d = Decoder::new();
    let mut pictures = 0;
    let mut out = Vec::new();
    for unit in split_access_units(stream) {
        match d.decode(unit) {
            Ok(Some(_)) => {
                pictures += 1;
                out.push(Ok(pictures));
            }
            Ok(None) => {}
            Err(e) => out.push(Err(e)),
        }
    }
    out
}

fn assert_lost_slice_reported(full: &[u8], lost: &[u8], bad: usize) {
    // Control: the intact stream decodes all five pictures, no error.
    assert_eq!(events(full), (1..=5).map(Ok).collect::<Vec<_>>());

    // The picture missing a slice is an error in its place; the pictures
    // before AND after it still decode.
    let mut want: Vec<Result<usize, DecodeError>> = (1..=bad).map(Ok).collect();
    want.push(Err(DecodeError::Truncated));
    want.extend((bad + 1..5).map(Ok));
    assert_eq!(events(lost), want);

    // The whole-stream entry point surfaces it too.
    assert_eq!(
        Decoder::new().decode_stream(lost).err(),
        Some(DecodeError::Truncated)
    );
}

#[test]
fn lost_middle_slice_cabac_is_reported() {
    assert_lost_slice_reported(
        include_bytes!("../../../tests/mslice_data/lost_slice_cabac_full.264"),
        include_bytes!("../../../tests/mslice_data/lost_slice_cabac.264"),
        2,
    );
}

#[test]
fn lost_middle_slice_cavlc_is_reported() {
    assert_lost_slice_reported(
        include_bytes!("../../../tests/mslice_data/lost_slice_cavlc_full.264"),
        include_bytes!("../../../tests/mslice_data/lost_slice_cavlc.264"),
        0,
    );
}
