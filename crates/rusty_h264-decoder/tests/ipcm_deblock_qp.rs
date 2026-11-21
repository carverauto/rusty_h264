// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! An `I_PCM` macroblock enters the loop filter with QP 0.
//!
//! Spec §8.7.2.2: when the macroblock containing p0 (or q0) is `I_PCM`, its
//! qPp (qPq) is 0, so its internal edges never filter and the edges it shares
//! with coded neighbours average 0 with the neighbour's QPy. Both entropy
//! paths stored the running slice QP for it instead, so every edge touching a
//! PCM block filtered at the wrong strength.
//!
//! The vectors are x264 0.165 (CLI), 64x64, three frames (IPP), High profile,
//! QP 20, deblocking on, over a synthetic checkerboard of 16x16 random-noise
//! cells and flat cells, which makes x264 code the noise cells as `I_PCM`:
//! `x264 --qp 20 --keyint 48 --bframes 0 --weightp 0 --psy-rd 0:0 --subme 7
//! --aq-mode 0 [--no-cabac]`. Noise never passes the filter's alpha/beta
//! tests, so each PCM payload was then replaced in place by a low-contrast
//! pattern of 4x4 blocks (values 3 apart) — raw PCM bytes are not entropy
//! coded, so the slices still parse identically, only the samples change.
//! The SEI was dropped. The reference YUV is ffmpeg 9.0's decode of the same
//! stream. Before the fix 7206 (CABAC) and 2374 (CAVLC) bytes differed
//! (two-sided gate: the vectors provably distinguish).

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
    assert!(ours == reference, "I_PCM deblock recon differs from ffmpeg");
}

#[test]
fn ipcm_deblock_qp_cabac_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/ipcm_data/ipcm_deblock_cabac.264"),
        include_bytes!("../../../tests/ipcm_data/ipcm_deblock_cabac_ref.yuv"),
        64,
        64,
        3,
    );
}

#[test]
fn ipcm_deblock_qp_cavlc_matches_ffmpeg() {
    assert_matches_ffmpeg(
        include_bytes!("../../../tests/ipcm_data/ipcm_deblock_cavlc.264"),
        include_bytes!("../../../tests/ipcm_data/ipcm_deblock_cavlc_ref.yuv"),
        64,
        64,
        3,
    );
}
