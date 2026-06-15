//! # Single channel "do-re-mi" example
//!
//! This code tests tone generation and frequency calculation on channel A by attempting to play the C major scale.

use std::thread;
use std::time::Duration;

use ay_psg::{
    audio::{AudioChannel, BuiltinEnvelopeShape, Envelope, Level},
    io::{IoPortMixerSettings},
    prelude::*,
};

struct DisplayWriter;
impl CommandOutput for DisplayWriter {
    fn execute(&mut self, command: Command) {
        println!("{}", command)
    }
}

const TEST_CHANNEL: AudioChannel = AudioChannel::A;
const DOREMI: [f32; 8] = [
    261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.26,
];

#[cfg(feature = "read")]
struct NullDriver;
#[cfg(feature = "read")]
impl Read for NullDriver {
    fn read<R: ay_psg::register::RegisterIndex>(&self, _: R) -> Result<u8, ay_psg::errors::Error> {
        Ok(0)
    }
}

fn main() {
    let out = DisplayWriter {};

    #[cfg(not(feature = "read"))]
    let mut chip = PSG::new(out, 2_000_000);
    #[cfg(feature = "read")]
    let mut chip = PSG::new(out, 2_000_000, ReadDriver(NullDriver {}));

    chip.setup_io_and_mixer(IoPortMixerSettings {
        tone_ch_a: true,
        tone_ch_b: true,
        tone_ch_c: true,
        ..Default::default()
    });

    chip.level(TEST_CHANNEL, Level::Fixed(0xF)).expect("Failed to set level");
    chip.set_envelope_shape(Envelope::Shape(BuiltinEnvelopeShape::Saw));
    loop {
        for f in DOREMI {
            chip.tone_hz(TEST_CHANNEL, f)
                .expect("Failed to set frequency");

            thread::sleep(Duration::from_millis(500));
        }
    }
}
