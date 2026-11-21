// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `second_chroma_qp_index_offset` must drive the Cr plane.
//!
//! A High-profile PPS carries two chroma QP offsets: `chroma_qp_index_offset`
//! for Cb and `second_chroma_qp_index_offset` for Cr (spec §7.4.2.2). Both the
//! residual scaling (§8.5.8) and the loop filter's chroma thresholds
//! (§8.7.2.2) take Cr's `QPc` from the second one. The decoder parsed it and
//! then used the Cb offset for both planes, so a stream with unequal offsets
//! decoded with a wrong Cr plane and no error.
//!
//! The vectors are x264 via ffmpeg 9.0, 64x64, six frames, IPPPPP, High
//! profile with the 8x8 transform, deblocking on, SEI removed:
//! `ffmpeg -f lavfi -i testsrc2=s=64x64:r=24:d=0.25 -c:v libx264 -profile:v
//! high -qp 30 -g 48 -bf 0 -x264-params weightp=0[:cabac=0] -bsf:v
//! h264_mp4toannexb -f h264`. x264 always writes equal offsets (-2, -2), so
//! the PPS was re-written with `second_chroma_qp_index_offset = 5`. A PPS
//! change does not alter how the slices parse, only how they reconstruct, so
//! the reference YUV is ffmpeg 9.0's decode of the SAME rewritten stream.
//! Before the fix every Cr plane differed (two-sided gate: the vectors
//! provably distinguish); Y and Cb were already exact.

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
    assert!(ours == reference, "Cr-offset recon differs from ffmpeg");
}

#[test]
fn second_chroma_qp_offset_cabac_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/chroma_qp_data/cr_offset_cabac.264"),
        include_bytes!("../../../tests/chroma_qp_data/cr_offset_cabac_ref.yuv"),
        64,
        64,
        6,
    );
}

#[test]
fn second_chroma_qp_offset_cavlc_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/chroma_qp_data/cr_offset_cavlc.264"),
        include_bytes!("../../../tests/chroma_qp_data/cr_offset_cavlc_ref.yuv"),
        64,
        64,
        6,
    );
}
