// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! I_8x8 corner availability across a slice boundary.
//!
//! An I_8x8 block's above-left sample has its own availability (neighbour D,
//! spec §6.4.11.2) and, when available, enters the reference-sample filter of
//! the top and left samples (§8.3.2.2.1). When a slice starts just right of a
//! macroblock's above-left neighbour, the top and left neighbours are in the
//! macroblock's slice but the corner is not; the decoder treated it as
//! available whenever top and left were, so every prediction of that block
//! read a sample from another slice.
//!
//! The vectors are x264 0.165 (CLI), High profile, 96x64 (6x4 macroblocks),
//! `--slice-max-mbs 7` so every slice after the first starts one macroblock
//! further into a row, over a synthetic test pattern (`testsrc2` plus temporal
//! noise, blurred), QP 26, SEI removed:
//! - CABAC, three intra pictures (`--keyint 1`);
//! - CAVLC (`--no-cabac`), six pictures, `--bframes 2`.
//!
//! The reference YUV is ffmpeg 9.0's decode. Before the fix 10 (CABAC) and 576
//! (CAVLC, the error propagating through prediction) bytes differed (two-sided
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
    assert!(ours == reference, "I_8x8 recon differs from ffmpeg");
}

#[test]
fn i8x8_corner_across_slices_cabac_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/mslice_data/i8x8_corner_cabac.264"),
        include_bytes!("../../../tests/mslice_data/i8x8_corner_cabac_ref.yuv"),
        96,
        64,
        3,
    );
}

#[test]
fn i8x8_corner_across_slices_cavlc_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/mslice_data/i8x8_corner_cavlc.264"),
        include_bytes!("../../../tests/mslice_data/i8x8_corner_cavlc_ref.yuv"),
        96,
        64,
        6,
    );
}
