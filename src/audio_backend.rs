use psimple::Simple;
use pulse::stream::Direction;
use pulse::sample::{Spec, Format};
use hound;
use hound::WavWriter;
use std::io::BufWriter;
use std::fs;

pub struct PulseBackend {
    simple: Simple
}

pub struct WavBackend {
    writer: WavWriter<BufWriter<fs::File>>
}

pub trait AudioBackEnd {
    fn write(&mut self, input: Vec<f32>);
}

impl AudioBackEnd for PulseBackend {
    fn write(&mut self, input: Vec<f32>) {
        let (ptr, parts, _) = input.into_raw_parts();
        let dat = unsafe { std::slice::from_raw_parts(ptr as *mut u8, parts * 4) };
        self.simple.write(dat).unwrap();
    }
}

impl PulseBackend {
    pub fn new() -> Self {
        let spec = Spec {
            format: Format::F32le,
            channels: 1,
            rate: 44100,
        };

        let s = Simple::new(
            None,                // Use the default server
            "Synthesizer",            // Our application’s name
            Direction::Playback, // We want a playback stream
            None,                // Use the default device
            "Music",             // Description of our stream
            &spec,               // Our sample format
            None,                // Use default channel map
            None                 // Use default buffering attributes
        ).unwrap();

        PulseBackend { simple: s }
    }
}

impl WavBackend {
    fn new() -> Self {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        WavBackend { writer: hound::WavWriter::create("output.wav", spec).unwrap() }
    }
}

impl AudioBackEnd for WavBackend {
    fn write(&mut self, input: Vec<f32>) {
        let amplitude = (i16::MAX as f32) * 0.75;
        for el in input {
            self.writer.write_sample((el * amplitude) as i16).unwrap();
        }
    }
}


pub fn create_backed(name: &str) -> Result<Box<dyn AudioBackEnd>, &str> {
    match name {
        "pulse" => Ok(Box::new(PulseBackend::new())),
        "wav" => Ok(Box::new(WavBackend::new())),
        _ => Err("Unknown backed")
    }
}