use crate::errors::Error;

/// Reference pitch of A=440.0Hz.
pub const REFERENCE_PITCH: f32 = 440.0;

/// One of the 3 audio channels available on PSG chips supported by the crate. This enum is used by code that requires any audio-related operations.
///
/// # Basic usage
///
/// ---
///
/// ```no_run
/// use ay_psg::audio::AudioChannel;
///
/// chip.tone_hz(AudioChannel::A, 110)?;
/// chip.tone_hz(AudioChannel::B, 220)?;
/// chip.tone_hz(AudioChannel::C, 440)?;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum AudioChannel {
    /// Audio Channel A
    A,
    /// Audio Channel B
    B,
    /// Audio Channel C
    C,
}

pub const AUDIO_CHANNELS: [AudioChannel; 3] = [AudioChannel::A, AudioChannel::B, AudioChannel::C];

impl AudioChannel {
    pub fn index(&self) -> usize {
        self.clone() as usize
    }
}

/// Output level configuration for an audio channel.
///
/// Used by [`level`](crate::chip::PSG::level) to select either:
/// - a fixed output level (`0..=15`)
/// - envelope-controlled output
#[derive(Debug, Clone, Copy)]
pub enum Level {
    Fixed(u8),
    EnvelopeControlled,
}

impl TryFrom<Level> for u8 {
    type Error = crate::errors::Error;

    fn try_from(value: Level) -> Result<Self, self::Error> {
        match value {
            Level::Fixed(n) => (n <= 0xF).then_some(n).ok_or(Error::LevelOutOfRange(n)),
            Level::EnvelopeControlled => Ok(0x10),
        }
    }
}

/// This enum covers all envelope types.
///
/// # Basic usage
///
/// ---
///
/// ```no_run
/// use ay_psg::{
///     audio::{BuiltinEnvelopeShape, Envelope},
///     prelude::*
/// };
///
/// let chip = PSG::new(_, _); // Replace with command output & clock
///
/// chip.set_envelope_shape(&Envelope::Shape(BuiltinEnvelopeShape::Saw));
/// chip.set_envelope_shape(&Envelope::Shape(BuiltinEnvelopeShape::Saw));
/// chip.set_envelope_shape(&Envelope::Shape(BuiltinEnvelopeShape::Saw));
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Envelope {
    /// One of the five builtin envelope shapes.
    Shape(BuiltinEnvelopeShape),
    /// One of the five builtin envelope shapes, inverted.
    InvertedShape(BuiltinEnvelopeShape),
    /// A byte to be written to register 0xD of the chip.
    ///
    /// ---
    ///
    /// From the datasheet:
    ///
    /// The envelope generator counts the envelope clock f_EA 32 times for each envelope pattern cycle.
    /// The envelope level is determined by the 5 bit output (E4~E0) of the counter. The shape of
    /// this envelope is created by increasing, decreasing, stopping, or repeating this counter. The
    /// shape is controlled by bits B3 ~ B0 of the register R_D.
    ///
    /// | B7 (MSB)  | B6  | B5  | B4  | B3  | B2  | B1  | B0  |
    /// |-----------|-----|-----|-----|-----|-----|-----|-----|
    /// | N/A       | N/A | N/A | N/A |CONT |ATT  |ALT  |HOLD |
    Raw(u8),
}

impl From<Envelope> for u8 {
    fn from(value: Envelope) -> Self {
        use Envelope::*; // for this scope only

        match value {
            Shape(builtin) => builtin as u8,
            InvertedShape(builtin) => (builtin as u8) ^ (0b0100),
            Raw(byte) => byte,
        }
    }
}

/// One of the five main builtin envelope shapes.
///
/// To invert the shape use [`Envelope::InvertedBuiltin`].
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BuiltinEnvelopeShape {
    /// Fade out and hold low
    FadeOut = 0b1001,
    /// Fade in and hold high
    FadeIn = 0b1101,
    /// Fade in then hold low
    Tooth = 0b1111,
    /// Fade in every repetition
    Saw = 0b1100,
    /// Alternate between fade out and fade in
    Triangle = 0b1110,
}

pub const ENVELOPE_SHAPES: [BuiltinEnvelopeShape; 5] = [
    BuiltinEnvelopeShape::FadeOut,
    BuiltinEnvelopeShape::FadeIn,
    BuiltinEnvelopeShape::Tooth,
    BuiltinEnvelopeShape::Saw,
    BuiltinEnvelopeShape::Triangle,
];

/// A helper enum for setting the envelope repetition frequency (f_e).
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeFrequency {
    /// Frequency in Hertz
    Hertz(u16),
    /// Frequency in beats per minute
    BeatsPerMinute(u16),
    /// Raw envelope frequncy value to be written to the chip
    Raw(u16),
}

impl EnvelopeFrequency {
    /// Returns the EnvelopeFrequency as a u16 value for registers 11 (LSB) and 12 (MSB).
    pub fn as_ep(self, clock_frequency: u32) -> Result<u16, Error> {
        match self {
            Self::Hertz(f_e) => {
                if f_e == 0 {
                    return Err(Error::DivisionByZero);
                }
                let period = clock_frequency / (256 * f_e as u32);
                period
                    .try_into()
                    .map_err(|_| Error::TonePeriodOutOfRange(period as u16))
            }
            Self::BeatsPerMinute(bpm) => {
                let hz = Self::Hertz(bpm).as_ep(clock_frequency)?;
                (60 * hz) // multiplying the period by 60 LOWERS the frequency, don't get confused by this
                    .try_into()
                    .map_err(|_| Error::TonePeriodOutOfRange(0))
            }
            Self::Raw(x) => Ok(x),
        }
    }
}
