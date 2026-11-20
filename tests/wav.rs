use std::io;

use linen::render::{WavError, quantize_pcm16, write_wav_header};
use linen::{RenderConfig, RenderError, compile, render};

#[test]
fn spec_6_3_wav_header_matches_independently_calculated_bytes() {
    let expected = [
        0x52, 0x49, 0x46, 0x46, 0x28, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6d, 0x74,
        0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x80, 0xbb, 0x00, 0x00, 0x00, 0x77,
        0x01, 0x00, 0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61, 0x04, 0x00, 0x00, 0x00,
    ];
    let mut actual = Vec::new();
    write_wav_header(&mut actual, 48_000, 2).expect("the small WAV header is valid");

    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 44);
}

#[test]
fn spec_6_3_pcm16_quantization_and_clipping_match_explicit_vectors() {
    let cases = [
        (-1.0, -32_767),
        (-0.5, -16_384),
        (0.0, 0),
        (0.5, 16_384),
        (1.0, 32_767),
        (-2.0, -32_767),
        (2.0, 32_767),
    ];

    for (input, expected) in cases {
        assert_eq!(
            quantize_pcm16(input).expect("finite input should quantize"),
            expected,
            "input {input}"
        );
    }

    for input in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        quantize_pcm16(input).expect_err("nonfinite input must be rejected");
    }
}

#[test]
fn spec_6_3_wav_size_overflow_is_rejected_before_writing() {
    let mut output = Vec::new();
    let error = write_wav_header(&mut output, 48_000, 2_147_483_630)
        .expect_err("the RIFF chunk size must overflow");

    assert!(matches!(error, WavError::SizeOverflow));
    assert!(output.is_empty());
}

#[test]
fn spec_9_beep_produces_exact_sample_count_and_wav_length() {
    let source = include_str!("../fixtures/valid/stage-01-beep.ln");
    let program = compile(source, RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("Beep should compile");
    let mut wav = Vec::new();
    let report = render(&program, &mut wav).expect("Beep should render");

    assert_eq!(report.samples, 96_000);
    assert_eq!(report.sample_rate, 48_000);
    assert_eq!(report.duration_seconds, 2.0);
    assert_eq!(wav.len(), 44 + 96_000 * 2);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 192_036);
    assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 192_000);
}

struct FailingWriter;

impl io::Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("intentional failure"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn spec_5_8_wav_writer_propagates_io_errors() {
    let program = compile("let out = 0; render 1ms;", RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("the constant program should compile");
    let error = render(&program, &mut FailingWriter).expect_err("the writer must fail");

    assert!(matches!(error, RenderError::Io(_)));
}

#[test]
fn spec_6_3_rendered_pcm_samples_are_little_endian() {
    let program = compile("let out = 0.5; render 0.00004s;", RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("the constant program should compile");
    let mut wav = Vec::new();
    let report = render(&program, &mut wav).expect("two constant samples should render");

    assert_eq!(report.samples, 2);
    assert_eq!(&wav[44..], [0x00, 0x40, 0x00, 0x40]);
}
