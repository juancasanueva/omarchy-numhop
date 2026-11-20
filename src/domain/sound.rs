//! Sound effects, built from simple tones instead of recorded files.
//!
//! Each effect is a short list of tones: a wave shape, a pitch that can glide, a start,
//! a length and a volume. They are mixed into 16-bit mono samples and wrapped as a WAV
//! file in memory, so the game ships no audio files and nothing needs a license.

use super::rng::Rng;

pub const SAMPLE_RATE: u32 = 22_050;

/// The loudest a mixed effect may get, as a fraction of full scale, to leave headroom.
const PEAK: f32 = 0.9;
/// Fade-in at the start of every tone, so no tone starts with a click.
const ATTACK: f32 = 0.004;
/// Silence added after the last tone ends.
const TAIL: f32 = 0.02;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sfx {
    Jump,
    Gem,
    Stomp,
    /// A Grumble caught the hero.
    Caught,
    /// The hero fell into a gap.
    Fall,
    /// A question appears.
    Ask,
    Right,
    Wrong,
    TimeUp,
    Checkpoint,
    WorldCleared,
    GameOver,
    MenuMove,
    MenuSelect,
}

impl Sfx {
    pub const ALL: [Sfx; 14] = [
        Sfx::Jump,
        Sfx::Gem,
        Sfx::Stomp,
        Sfx::Caught,
        Sfx::Fall,
        Sfx::Ask,
        Sfx::Right,
        Sfx::Wrong,
        Sfx::TimeUp,
        Sfx::Checkpoint,
        Sfx::WorldCleared,
        Sfx::GameOver,
        Sfx::MenuMove,
        Sfx::MenuSelect,
    ];

    /// The tones that make up this effect.
    pub fn tones(self) -> Vec<Tone> {
        use Wave::*;
        match self {
            Sfx::Jump => vec![Tone::slide(Square, 0.0, 0.16, 300.0, 620.0, 0.22)],
            Sfx::Gem => vec![Tone::note(Square, 0.0, 0.07, B5, 0.2), Tone::note(Square, 0.07, 0.25, E6, 0.2)],
            Sfx::Stomp => vec![
                Tone::slide(Square, 0.0, 0.14, 420.0, 140.0, 0.3),
                Tone::note(Noise, 0.0, 0.06, 0.0, 0.15),
            ],
            Sfx::Caught => vec![
                Tone::slide(Triangle, 0.0, 0.55, 520.0, 130.0, 0.6),
                Tone::slide(Square, 0.0, 0.55, 260.0, 65.0, 0.12),
            ],
            Sfx::Fall => vec![Tone::slide(Triangle, 0.0, 0.7, 900.0, 120.0, 0.6)],
            Sfx::Ask => vec![Tone::note(Triangle, 0.0, 0.12, A5, 0.5), Tone::note(Triangle, 0.09, 0.3, E6, 0.5)],
            Sfx::Right => vec![
                Tone::note(Triangle, 0.0, 0.1, C5, 0.5),
                Tone::note(Triangle, 0.09, 0.1, E5, 0.5),
                Tone::note(Triangle, 0.18, 0.1, G5, 0.5),
                Tone::note(Triangle, 0.27, 0.35, C6, 0.5),
                Tone::note(Square, 0.27, 0.35, C6, 0.08),
            ],
            Sfx::Wrong => vec![
                Tone::note(Square, 0.0, 0.18, 233.08, 0.22),
                Tone::slide(Square, 0.2, 0.4, 174.61, 160.0, 0.22),
            ],
            Sfx::TimeUp => vec![
                Tone::note(Square, 0.0, 0.08, A5, 0.18),
                Tone::note(Square, 0.13, 0.08, A5, 0.18),
                Tone::slide(Square, 0.26, 0.35, E5, 330.0, 0.18),
            ],
            Sfx::Checkpoint => vec![Tone::note(Triangle, 0.0, 0.09, G5, 0.45), Tone::note(Triangle, 0.08, 0.25, C6, 0.45)],
            Sfx::WorldCleared => vec![
                Tone::note(Triangle, 0.0, 0.13, C5, 0.45),
                Tone::note(Triangle, 0.13, 0.13, E5, 0.45),
                Tone::note(Triangle, 0.26, 0.13, G5, 0.45),
                Tone::note(Triangle, 0.39, 0.55, C6, 0.45),
                Tone::note(Square, 0.0, 0.13, C4, 0.07),
                Tone::note(Square, 0.13, 0.13, E4, 0.07),
                Tone::note(Square, 0.26, 0.13, G4, 0.07),
                Tone::note(Square, 0.39, 0.55, C5, 0.07),
            ],
            Sfx::GameOver => vec![
                Tone::note(Triangle, 0.0, 0.28, G4, 0.55),
                Tone::note(Triangle, 0.28, 0.28, 311.13, 0.55),
                Tone::slide(Triangle, 0.56, 0.7, C4, 245.0, 0.55),
            ],
            Sfx::MenuMove => vec![Tone::note(Square, 0.0, 0.035, 1200.0, 0.08)],
            Sfx::MenuSelect => vec![Tone::note(Square, 0.0, 0.045, 880.0, 0.12), Tone::note(Square, 0.045, 0.07, 1320.0, 0.12)],
        }
    }

    /// This effect as a complete WAV file.
    pub fn wav(self) -> Vec<u8> {
        wav(&render(&self.tones()))
    }
}

// Note pitches in hertz, equal temperament with A4 = 440.
const C4: f32 = 261.63;
const E4: f32 = 329.63;
const G4: f32 = 392.00;
const C5: f32 = 523.25;
const E5: f32 = 659.25;
const G5: f32 = 783.99;
const A5: f32 = 880.00;
const B5: f32 = 987.77;
const C6: f32 = 1046.50;
const E6: f32 = 1318.51;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wave {
    Square,
    Triangle,
    /// White noise; the pitch is ignored.
    Noise,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tone {
    pub wave: Wave,
    /// Seconds from the start of the effect.
    pub start: f32,
    /// Seconds.
    pub length: f32,
    /// Pitch at the start and at the end, in hertz; it glides between them.
    pub from: f32,
    pub to: f32,
    /// Loudness before mixing, from 0 to 1.
    pub volume: f32,
}

impl Tone {
    pub fn note(wave: Wave, start: f32, length: f32, hz: f32, volume: f32) -> Self {
        Self::slide(wave, start, length, hz, hz, volume)
    }

    pub fn slide(wave: Wave, start: f32, length: f32, from: f32, to: f32, volume: f32) -> Self {
        Self { wave, start, length, from, to, volume }
    }

    fn end(&self) -> f32 {
        self.start + self.length
    }
}

/// Mixes the tones into 16-bit samples at [`SAMPLE_RATE`]. Each tone fades in quickly
/// and dies away to silence by its end; the mix is scaled down only if it would clip.
pub fn render(tones: &[Tone]) -> Vec<i16> {
    let rate = SAMPLE_RATE as f32;
    let end = tones.iter().map(Tone::end).fold(0.0, f32::max);
    let mut mix = vec![0.0f32; ((end + TAIL) * rate).ceil() as usize];

    for (i, tone) in tones.iter().enumerate() {
        let first = (tone.start * rate).round() as usize;
        let count = (tone.length * rate).round() as usize;
        let mut noise = Rng::new(i as u64 + 1);
        let mut phase = 0.0f32;
        for (n, out) in mix.iter_mut().skip(first).take(count).enumerate() {
            let t = n as f32 / rate;
            let progress = n as f32 / count as f32;
            // Glide in pitch space, so a slide sounds even all the way through.
            let hz = tone.from * (tone.to / tone.from.max(1.0)).max(0.0).powf(progress);
            phase = (phase + hz / rate).fract();
            let value = match tone.wave {
                Wave::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
                Wave::Noise => (noise.next_u64() >> 40) as f32 / (1u64 << 23) as f32 - 1.0,
            };
            let attack = (t / ATTACK).min(1.0);
            let decay = (1.0 - progress).powf(1.3);
            *out += value * tone.volume * attack * decay;
        }
    }

    let peak = mix.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let gain = if peak > PEAK { PEAK / peak } else { 1.0 };
    mix.iter().map(|s| (s * gain * i16::MAX as f32).round() as i16).collect()
}

/// Wraps mono 16-bit samples at [`SAMPLE_RATE`] in a WAV (RIFF, PCM) file.
pub fn wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // size of the format block
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // bytes per second
    out.extend_from_slice(&2u16.to_le_bytes()); // bytes per sample frame
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_effect_is_short_audible_and_never_clips() {
        let limit = (PEAK * i16::MAX as f32).ceil() as i32;
        for sfx in Sfx::ALL {
            let samples = render(&sfx.tones());
            let seconds = samples.len() as f32 / SAMPLE_RATE as f32;
            assert!(seconds > 0.03 && seconds <= 1.5, "{sfx:?} lasts {seconds}s");
            let peak = samples.iter().map(|s| (*s as i32).abs()).max().unwrap();
            assert!(peak <= limit, "{sfx:?} peaks at {peak}");
            assert!(peak > i16::MAX as i32 / 20, "{sfx:?} is nearly silent ({peak})");
        }
    }

    #[test]
    fn every_effect_starts_and_ends_in_silence() {
        // A jump from silence to a loud sample is heard as a click.
        let quiet = i16::MAX / 100;
        for sfx in Sfx::ALL {
            let samples = render(&sfx.tones());
            assert!(samples[0].abs() <= quiet, "{sfx:?} starts with a click");
            assert!(samples.last().unwrap().abs() <= quiet, "{sfx:?} ends with a click");
        }
    }

    #[test]
    fn every_effect_sounds_different() {
        let distinct: HashSet<Vec<u8>> = Sfx::ALL.iter().map(|s| s.wav()).collect();
        assert_eq!(distinct.len(), Sfx::ALL.len());
    }

    #[test]
    fn rendering_is_repeatable() {
        assert_eq!(Sfx::Stomp.wav(), Sfx::Stomp.wav());
    }

    #[test]
    fn wav_header_describes_the_samples() {
        let bytes = wav(&[0, 1, -1]);
        let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        let u16_at = |i: usize| u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap());
        assert_eq!(bytes.len(), 44 + 6);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u32_at(4), bytes.len() as u32 - 8);
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(u16_at(20), 1, "PCM");
        assert_eq!(u16_at(22), 1, "mono");
        assert_eq!(u32_at(24), SAMPLE_RATE);
        assert_eq!(u16_at(34), 16, "bits per sample");
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32_at(40), 6);
        assert_eq!(&bytes[44..], &[0, 0, 1, 0, 0xFF, 0xFF]);
    }
}
