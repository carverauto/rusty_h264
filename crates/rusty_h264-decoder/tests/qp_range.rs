// Dev/test target, not shipped: a panic here IS the diagnostic (that is what an
// assertion is). The workspace's unwrap/expect/panic denials exist to keep them
// off the decoder's untrusted-input path, so they are relaxed for this file.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Out-of-range quantiser fields are malformed streams, not values to clamp.
//!
//! Spec §7.4.2.2 bounds `pic_init_qp_minus26` to -26..=25 and
//! `chroma_qp_index_offset` / `second_chroma_qp_index_offset` to -12..=12;
//! §7.4.3 bounds SliceQPY = 26 + pic_init_qp_minus26 + slice_qp_delta to
//! 0..=51 (8-bit video). The decoder clamped the slice QP and the chroma
//! index into range and decoded on, i.e. reconstructed a picture the stream
//! does not describe, with no error. ffmpeg rejects the chroma offsets at the
//! PPS and the slice QP at the slice header ("QP 52 out of range").
//!
//! The vectors are one x264 stream via ffmpeg 9.0 (32x32, three frames, High,
//! CABAC; `qp_slice_m1` is the CAVLC twin), SEI removed, with a single field
//! rewritten each: PPS chroma offset 13, PPS second chroma offset -13, PPS
//! `pic_init_qp_minus26` 26 and -27, slice QP 52 (CABAC) and -1 (CAVLC).
//! `qp_ok` is the stream unmodified and `qp_offsets_at_limits` carries the
//! extreme legal offsets (12, -12): both must still decode. Before the fix
//! every malformed stream decoded to three frames with no error.

use rusty_h264_decoder::{DecodeError, Decoder};

fn decode(stream: &[u8]) -> Result<usize, DecodeError> {
    Decoder::new().decode_stream(stream).map(|f| f.len())
}

fn assert_rejected(stream: &[u8], what: &str) {
    match decode(stream) {
        Err(DecodeError::Unsupported(msg)) => assert!(msg.contains(what), "{msg}"),
        other => panic!("expected a range error naming {what:?}, got {other:?}"),
    }
}

#[test]
fn in_range_quantiser_fields_decode() {
    assert_eq!(
        decode(include_bytes!("../../../tests/qp_range_data/qp_ok.264")),
        Ok(3)
    );
    assert_eq!(
        decode(include_bytes!(
            "../../../tests/qp_range_data/qp_offsets_at_limits.264"
        )),
        Ok(3)
    );
}

#[test]
fn chroma_qp_offsets_out_of_range_are_rejected() {
    assert_rejected(
        include_bytes!("../../../tests/qp_range_data/qp_cb_offset_13.264"),
        "chroma QP index offset",
    );
    assert_rejected(
        include_bytes!("../../../tests/qp_range_data/qp_cr_offset_m13.264"),
        "chroma QP index offset",
    );
}

#[test]
fn pic_init_qp_out_of_range_is_rejected() {
    assert_rejected(
        include_bytes!("../../../tests/qp_range_data/qp_init_52.264"),
        "pic_init_qp_minus26",
    );
    assert_rejected(
        include_bytes!("../../../tests/qp_range_data/qp_init_m1.264"),
        "pic_init_qp_minus26",
    );
}

#[test]
fn slice_qp_out_of_range_is_rejected() {
    assert_rejected(
        include_bytes!("../../../tests/qp_range_data/qp_slice_52.264"),
        "slice QP",
    );
    assert_rejected(
        include_bytes!("../../../tests/qp_range_data/qp_slice_m1.264"),
        "slice QP",
    );
}
