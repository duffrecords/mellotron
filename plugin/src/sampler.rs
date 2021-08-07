use dirs::home_dir;
use lv2::prelude::*;
use regex::Regex;
use std::fs::File;
use std::f32::consts::PI;

const START_NOTE: usize = 57; // A2
const END_NOTE: usize = 89;   // F5
const NORMALIZE_OFFSET: f32 = 0.1;

fn root_mean_square(vec: Vec<f32>) -> f32 {
    if vec.len() == 0 { return 0.0 }
    if vec.len() == 1 { return vec[0] }
    let sum_squares = vec.iter().fold(0.0, |acc, &x| acc + (f64::from(x)).powi(2));
    return (sum_squares/(vec.len() as f64)).sqrt() as f32;
}

fn averaged_sum(vec: Vec<f32>) -> f32 {
    if vec.len() == 0 { return 0.0 }
    if vec.len() == 1 { return vec[0] }
    let sum = vec.iter().fold(0.0, |acc, &x| acc + f64::from(x));
    // // println!("{} * {} = {}", sum, (sum_squares/(vec.len() as f64)).sqrt(), (sum * (sum_squares/(vec.len() as f64)).sqrt()) as f32);
    // return (sum * (sum_squares/(vec.len() as f64)).sqrt()) as f32;
    return sum as f32
}

fn crossfade(a: f32, b: f32, ratio: f32) -> f32 {
    let angle = ratio * PI/2.0;
    return a * angle.cos() + b * angle.sin()
}

#[derive(Clone, Debug)]
pub struct Sample {
    name: wmidi::Note,
    data: Vec<f32>,
}

impl Sample {
    pub fn new(name: wmidi::Note, data: Vec<f32>) -> Self {
        Self {
            name,
            data,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Voice {
    name: wmidi::Note,
    sample: Vec<f32>,
    gain: f32,
    active: bool,
    frame: usize,
}

impl Voice {
    pub fn new(name: wmidi::Note, sample: Vec<f32>) -> Self
    {
        Self {
            name,
            sample,
            gain: 1.0,
            active: true,
            frame: 0,
        }
    }

    pub fn from(sample: Sample) -> Self {
        Self {
            name: sample.name,
            sample: sample.data.clone(),
            gain: 1.0,
            active: true,
            frame: 0,
        }
    }

    pub fn value(&self) -> f32 {
        // return the instantaneous value of the current sample
        let val = self.sample[self.frame];
        val
    }

    pub fn process(&mut self) -> Option<f32> {
        // let mut sum = 0.0;
        let mut out = 0.0;
        let mut active = false;
        //for b in buf.iter_mut() {
            if self.active {
                // sum += self.value() * self.gain * 0.7071;
                if self.gain < 1.0 {
                    if self.gain > 0.01 {
                        self.gain -= 0.01;
                    } else {
                        self.gain = 0.0;
                        self.active = false;
                        println!("note {:?} faded out", self.name);
                    }
                }
                out = self.value() * self.gain;
                active = true;
            }
            if self.frame == self.sample.len() - 1 {
                self.active = false;
                self.frame = 0;
            } else {
                self.frame += 1;
            }
            // *o += sum * 0.7071 * NORMALIZE_OFFSET
        //}
        if active {
            Some(out)
        } else {
            None
        }
    }

    pub fn all_notes_off(&mut self) {
        self.gain = 0.0;
        self.frame = 0;
        // for voice in &mut self.voices {
        //     voice.envelope_state = envelopes::State::Release(0);
        //     voice.release_start_gain = voice.last_envelope_gain;
        // }
    }

    fn start_playing(&mut self) {
        self.gain = 1.0;
        self.frame = 0;
        self.active = true;
    }

}


fn load_samples(instrument: std::string::String) -> Vec<Sample> {
    // read sample data from all the files in a bank
    let mut samples: Vec<Sample> = Vec::with_capacity(35);
    let re = Regex::new(r"/[A-G]b?").unwrap();
    let home = match home_dir() {
        Some(h) => h,
        _ => std::path::PathBuf::from("."),
    };
    for n in START_NOTE..END_NOTE + 1 {
        let midi_note = unsafe { wmidi::Note::from_u8_unchecked(n as u8) };
        let note_name = re.replace(midi_note.to_str(), "");
        let filename = format!("{}.wav", note_name);
        let mut inp_file = File::open(home.as_path().join(".lv2/mellotron.lv2/samples").join(instrument.clone()).join(filename)).unwrap();
        let (header, data) = wav::read(&mut inp_file).unwrap();
        let values = match header.bits_per_sample {
            32 => data.as_thirty_two_float().unwrap().to_owned().into_iter().map(|d| f32::from(d)).collect(),
            24 => data.as_twenty_four().unwrap().to_owned().into_iter().map(|d| d as f32).collect(),
            16 => data.as_sixteen().unwrap().to_owned().into_iter().map(|d| f32::from(d)/f32::from(i16::MAX)).collect(),
            8 => data.as_eight().unwrap().to_owned().into_iter().map(|d| f32::from(d)/f32::from(u8::MAX/2)).collect(),
            _ => Vec::new(),
        };
        // let max = values.iter().cloned().fold(0./0., f32::max);
        // let min = values.iter().cloned().fold(0./0., f32::min);
        samples.push(Sample::new(midi_note, values));
        // println!("loaded {}\tmin/max: {}/{}", note_name, min, max);
    }
    // println!("loaded {} samples for {}", samples.len(), instrument);
    samples
}


#[derive(Debug)]
pub enum SamplerError {
}

// sampler engine
pub struct Instrument {
    name: std::string::String,
    patch: urids::Patch,
    samples: Vec<Sample>,
    voices: Vec<Voice>,
}

impl Instrument {
    pub fn new(mut patch: urids::Patch) -> Self {
        let samples = load_samples(patch.to_string().to_lowercase().replace(" ", "_").replace("(", "").replace(")", ""));
        println!("DEBUG loading {}", patch.to_string());
        Self {
            name: patch.to_string(),
            patch: patch,
            samples: samples,
            voices: Vec::new(),
        }
    }
    pub fn prev_patch(&mut self) {
        self.patch.prev();
        self.name = self.patch.to_string();
        self.samples = load_samples(self.patch.to_string().to_lowercase().replace(" ", "_"));
    }
    pub fn next_patch(&mut self) {
        self.patch.next();
        self.name = self.patch.to_string();
        self.samples = load_samples(self.patch.to_string().to_lowercase().replace(" ", "_"));
    }
}

pub struct Sampler {
    pub a: Instrument,
    pub b: Instrument,
    pub mix: f32,
}

impl Sampler {
    pub fn new(mut patch_a: urids::Patch, mut patch_b: urids::Patch) -> Result<Sampler, SamplerError> {
        // let voices = load_samples(patch.to_string().to_lowercase().replace(" ", "_").replace("(", "").replace(")", ""));
        println!("DEBUG loading {} and {}", patch_a.to_string(), patch_b.to_string());
        Ok(
            Sampler {
                a: Instrument::new(urids::Patch::new(urids::PatchName::MkIIFlute)),
                b: Instrument::new(urids::Patch::new(urids::PatchName::StringSection)),
                mix: 0.0,
            }
        )
    }

    pub fn process(&mut self, out: &mut [f32], mix: f32) {
        if out.len() == 0 {
            return;
        }
        // for v in &mut self.voices {
        //     v.process(out);
        // }
        for o in out.iter_mut() {
            let slice_a: Vec<f32> = self.a.voices.iter_mut().map(|v| v.process()).filter(|v| v.is_some()).map(|v| v.unwrap()).collect();
            let slice_b: Vec<f32> = self.b.voices.iter_mut().map(|v| v.process()).filter(|v| v.is_some()).map(|v| v.unwrap()).collect();
            // *o = root_mean_square(slice);
            *o = crossfade(averaged_sum(slice_a), averaged_sum(slice_b), mix);
            // *o = slice.iter().sum();
            // for v in self.voices.iter_mut() {
            //     *o += v.process();
            // }
            // if let Some(pos) = self.a.voices.iter().position(|v| v.active == false) {
            //     println!("removing {:?}", self.a.voices[pos].name);
            //     self.a.voices.remove(pos);
            // }
            // if let Some(pos) = self.b.voices.iter().position(|v| v.active == false) {
            //     self.b.voices.remove(pos);
            // }
        }
        // let buffers = &mut self.voices.iter().map(|v| v.process(out.len()));
        // for i in 0..out.len() {
        //     for b in buffers {
        //         out[i] += b[i];
        //     }
        // }
    }

    pub fn midi_event(&mut self, midi_msg: &wmidi::MidiMessage) {
        match midi_msg {
            wmidi::MidiMessage::NoteOn(_ch, note, vel) => self.note_on(*note),
            wmidi::MidiMessage::NoteOff(_ch, note, _vel) => self.note_off(*note),
            wmidi::MidiMessage::ControlChange(_ch, cnum, cval) => {
                // println!("DEBUG control change {:?} {:?}", cnum, cval);
                // if u8::from(*cnum) == 14 {
                //     self.mix = f32::from(u8::from(*cval))/f32::from(u8::from(wmidi::U7::MAX));
                // }
            }
            _ => {}
        }
    }

    fn note_on(&mut self, note: wmidi::Note) {
        println!("DEBUG note on {:?}", note);
        let n = u8::from(note) as usize;
        if n >= START_NOTE && n <= END_NOTE {
            if let Some(pos) = self.a.voices.iter().position(|v| v.name == note) {
                self.a.voices[pos].start_playing();
                self.b.voices[pos].start_playing();
            } else {
                self.a.voices.push(Voice::new(note, self.a.samples[n-START_NOTE].data.clone()));
                self.b.voices.push(Voice::new(note, self.b.samples[n-START_NOTE].data.clone()));
            }
        }
        println!("{} voices are playing", self.a.voices.len());
    }

    fn note_off(&mut self, note: wmidi::Note) {
        println!("DEBUG note off {:?}", note);
        let n = u8::from(note) as usize;
        if n >= START_NOTE && n <= END_NOTE {
            if let Some(pos) = self.a.voices.iter().position(|v| v.name == note) {
                // self.a.voices[n].active = false;
                self.a.voices[pos].gain -= 0.01;
                // self.b.voices[n].active = false;
                self.b.voices[pos].gain -= 0.01;
            }
        }
    }

    pub fn fadeout(&mut self) {
        for v in &mut self.a.voices {
            v.all_notes_off();
        }
        for v in &mut self.b.voices {
            v.all_notes_off();
        }
    }

    pub fn fadeout_finished(&self) -> bool {
        !self.a.voices.iter().any(|v| v.active) && !self.b.voices.iter().any(|v| v.active)
    }

    pub fn dummy() -> Result<Sampler, SamplerError> {
        Sampler::new(
            urids::Patch::new(urids::PatchName::MkIIFlute),
            urids::Patch::new(urids::PatchName::StringSection),
        )
    }

}

