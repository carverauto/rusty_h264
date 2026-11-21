// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Each macroblock is deblocked with ITS OWN slice's filter parameters.
//!
//! `disable_deblocking_filter_idc` and the alpha/beta offsets are slice header
//! fields, and the loop filter applies those of the slice containing the
//! macroblock whose edges are being filtered (spec §8.7, §8.7.2.2). The
//! decoder filtered each completed macroblock row with the parameters of the
//! slice being decoded when the row completed, latched filtering off for the
//! rest of the picture once any slice disabled it, and finished the picture
//! with the LAST slice's settings, so a picture whose slices disagree decoded
//! wrong with no error.
//!
//! The vectors are x264 0.165 (CLI) multi-slice streams, 96x64 (6x4
//! macroblocks), six frames, over a synthetic test pattern (`testsrc2` plus
//! temporal noise, blurred), QP 32, SEI removed; every slice header was then
//! rewritten with its own deblocking fields (x264 writes the same ones in every
//! slice). The fields do not affect parsing, so the reference YUV is ffmpeg
//! 9.0's decode of the same rewritten stream.
//!
//! - CABAC: Main, `--bframes 2 --slice-max-mbs 3` (I/P/B, slices start mid
//!   row), slice k gets idc `[0, 1, 2]`, alpha `[3, 0, -2]`, beta `[-1, 0, 4]`
//!   (`*_div2` units) at `k % 3`.
//! - CAVLC: Baseline, `--slice-max-mbs 5` (I/P), idc `[1, 0, 0]`, alpha
//!   `[0, 4, -4]`, beta `[0, -3, 5]`.
//!
//! Before the fix 20719 (CABAC) and 19230 (CAVLC) bytes differed (two-sided
//! gate: the vectors provably distinguish).

use rusty_h264_decoder::Decoder;

/// Decode `stream` and require it to match ffmpeg's decode byte-for-byte.
fn assert_matches_ffmpeg(stream: &[u8], reference: &[u8], w: usize, h: usize, n: usize) {
    let frames = Decoder::new().decode_stream(stream).expect("decode");
    assert_eq!(frames.len(), n);

    let mut ours = Vec::with_capacity(reference.len());
    for f in &frames {
        assert_eq!((f.width, f.height), (w, h));
        ours.extend_from_slice(&f.y);
        ours.extend_from_slice(&f.u);
        ours.extend_from_slice(&f.v);
    }
    assert!(
        ours == reference,
        "per-slice deblocking recon differs from ffmpeg"
    );
}

#[test]
fn per_slice_deblocking_cabac_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/mslice_data/mslice_deblock_cabac.264"),
        include_bytes!("../../../tests/mslice_data/mslice_deblock_cabac_ref.yuv"),
        96,
        64,
        6,
    );
}

#[test]
fn per_slice_deblocking_cavlc_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/mslice_data/mslice_deblock_cavlc.264"),
        include_bytes!("../../../tests/mslice_data/mslice_deblock_cavlc_ref.yuv"),
        96,
        64,
        6,
    );
}
