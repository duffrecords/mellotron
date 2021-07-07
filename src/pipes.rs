use iterpipes::*;
use lv2::prelude::*;
use regex::Regex;
use std::fs::File;
use std::path::Path;

const START_NOTE: usize = 57; // A2
const END_NOTE: usize = 89;   // F5
const NORMALIZE_OFFSET: f32 = 0.1;

fn _root_mean_square(vec: Vec<f32>) -> f32 {
    let sum_squares = vec.iter().fold(0.0, |acc, &x| acc + x.powi(2));
    return ((sum_squares as f32)/(vec.len() as f32)).sqrt();
}

fn load_samples(instrument: std::string::String) -> Vec<Note> {
    // read sample data from all the files in a bank
    let mut samples: Vec<Note> = Vec::with_capacity(35);
    let re = Regex::new(r"/[A-G]b?").unwrap();
    for n in START_NOTE..END_NOTE + 1 {
        let midi_note = unsafe { wmidi::Note::from_u8_unchecked(n as u8) };
        let note_name = re.replace(midi_note.to_str(), "");
        let filename = format!("{}.wav", note_name);
        let mut inp_file = File::open(Path::new("samples").join(instrument.clone()).join(filename)).unwrap();
        let (header, data) = wav::read(&mut inp_file).unwrap();
        let values = match header.bits_per_sample {
            32 => data.as_thirty_two_float().unwrap().to_owned().into_iter().map(|d| f32::from(d)).collect(),
            24 => data.as_twenty_four().unwrap().to_owned().into_iter().map(|d| d as f32).collect(),
            16 => data.as_sixteen().unwrap().to_owned().into_iter().map(|d| f32::from(d)).collect(),
            8 => data.as_eight().unwrap().to_owned().into_iter().map(|d| f32::from(d)).collect(),
            _ => Vec::new(),
        };
        // let max = values.iter().cloned().fold(0./0., f32::max);
        // let min = values.iter().cloned().fold(0./0., f32::min);
        samples.push(Note::new(midi_note, values));
        // println!("loaded {}\tmin/max: {}/{}", note_name, min, max);
    }
    // println!("loaded {} samples for {}", samples.len(), instrument);
    samples
}

#[derive(Clone, Copy, Debug)]
pub struct NoteUpdate {
    pub note: wmidi::Note,
    pub onoff: bool,
    pub velocity: wmidi::U7,
}

#[derive(Clone, Debug)]
pub struct Note {
    name: wmidi::Note,
    sample: Vec<f32>,
    gain: f32,
    active: bool,
    frame: usize,
}

impl Note {
    pub fn new(name: wmidi::Note, sample: Vec<f32>) -> Self
    {
        Self {
            name: name,
            sample: sample,
            gain: 1.0,
            active: false,
            frame: 0,
        }
    }
    pub fn value(&self) -> f32 {
        // return the instantaneous value of the current sample
        let val = self.sample[self.frame];
        val
    }
}

pub struct Voice {
    name: std::string::String,
    notes: Vec<Note>,
    // max_sum: f32,
    // max_avg: f32,
    // max_rms: f32,
}

impl Voice {
    pub fn new(instrument: std::string::String) -> Self
    {
        Self {
            name: instrument.clone(),
            notes: load_samples(instrument.to_lowercase().replace(" ", "_")),
            // max_sum: 0.0,
            // max_avg: 0.0,
            // max_rms: 0.0,
        }
    }
}

impl Pipe for Voice
// where
//     T: Copy,
{
    type InputItem = Option<NoteUpdate>;
    type OutputItem = f32;

    fn next(&mut self, update: Option<NoteUpdate>) -> f32 {
        if let Some(update) = update {
            // convert to note number
            let mut n = u8::from(update.note) as usize;
            if n >= START_NOTE && n <= END_NOTE {
                n -= START_NOTE;
                // set note to be active or inactive
                if update.onoff {
                    self.notes[n].frame = 0;
                    self.notes[n].gain = 1.0;
                    self.notes[n].active = true;
                } else {
                    self.notes[n].gain -= 0.01;
                }
            }
        }
        // sum the audio of all active notes and advance each by one sample or deactivate it when finished playing
        let mut active_samples = Vec::new();
        let total = self.notes.iter().filter(|x| x.active).count() as f32;
        if total > 0.0 {
            let mut sum = 0.0;
            for i in 0..self.notes.len() {
                if self.notes[i].active {
                    // sum together the current frame of all active notes
                    sum += self.notes[i].value() * self.notes[i].gain * 0.7071;
                    active_samples.push(self.notes[i].value() * self.notes[i].gain);
                    if self.notes[i].gain < 1.0 {
                        if self.notes[i].gain > 0.01 {
                            self.notes[i].gain -= 0.01;
                        } else {
                            self.notes[i].gain = 0.0;
                            self.notes[i].active = false;
                        }
                    }
                }
                if self.notes[i].frame == self.notes[i].sample.len() - 1 {
                    // reset sample if it has played to the end
                    self.notes[i].active = false;
                    self.notes[i].frame = 0;
                } else {
                    // advance frame
                    self.notes[i].frame += 1;
                }
            }
            // println!("{} / {} = {}", sum, total, sum / total);
            // if sum > self.max_sum { self.max_sum = sum }
            // let avg = sum / total;
            // if avg > self.max_avg { self.max_avg = avg }
            // let rms = root_mean_square(active_samples);
            // if rms > self.max_rms { self.max_rms = rms }
            // if let Some(update) = update {
            //     if !update.onoff { println!("sum: {}\tavg: {}\trms: {}", self.max_sum, self.max_avg * 0.1, self.max_rms); self.max_sum = 0.0; self.max_avg = 0.0; self.max_rms = 0.0 }
            // }
            // (sum / total) * 0.1
            // rms * NORMALIZE_OFFSET
            sum * 0.7071 * NORMALIZE_OFFSET
        } else {
            0.0
        }
    }
}

impl ResetablePipe for Voice
// where
//     T: Copy,
{
    fn reset(&mut self) {}
}

// The `EventAtomizer` wraps an iterator over events and transforms them into frames, which either contain an event or don't. This iterator will be the atom event iterator later, but for now, it's good to be generic.
//
// Internally, it stores the next event of the event sequence. Every time `next` is called, this counter is increased and once it hits this next event, it is yielded and the next "next event" is retrieved. This is continued as long as the sequence contains events. Once it is depleted, this pipe only emits `None`s.
//
// Since every frame can only contain one event and frames must be emitted chronologically, it drops every event that has the same or an earlier timestamp than a previous event.
pub struct EventAtomizer<T, I>
where
    I: Iterator<Item = (usize, T)>,
{
    sequence: I,
    next_event: Option<(usize, T)>,
    index: usize,
}

impl<T, I> EventAtomizer<T, I>
where
    I: Iterator<Item = (usize, T)>,
{
    pub fn new(sequence: I) -> Self {
        let mut instance = Self {
            sequence,
            next_event: None,
            index: 0,
        };
        instance.retrieve_next_event();
        instance
    }

    fn retrieve_next_event(&mut self) {
        self.next_event = None;
        if let Some((index, item)) = self.sequence.next() {
            self.next_event = Some((index, item));
        }
    }
}

impl<T, I> Pipe for EventAtomizer<T, I>
where
    I: Iterator<Item = (usize, T)>,
{
    type InputItem = ();
    type OutputItem = Option<T>;

    fn next(&mut self, _: ()) -> Option<T> {
        match self.next_event.take() {
            Some((_event_index, event_atom)) => {
                self.retrieve_next_event();
                Some(event_atom)
            }
            None => None,
        }
    }
}

#[test]
fn test_atomizer() {
    let events: Box<[(usize, u32)]> = Box::new([(4, 1), (10, 5)]);
    let mut pipe = EventAtomizer::new(events.iter().cloned());

    for i in 0..15 {
        let output = pipe.next(());
        match i {
            4 => assert_eq!(Some(1), output),
            10 => assert_eq!(Some(5), output),
            _ => assert_eq!(None, output),
        }
    }
}

// In the final plugin, the `EventAtomizer` emits `Option<UnidentifiedAtom>`s, which might be any atom at all, and the `PulseGenerator` consumes `PulseInput`s. The `EventReader` bridges the gap between these two pipes by identifying the atom, reading it and emitting an appropriate `PulseInput`.
//
// This is the only pipe that isn't tested since creating a testbed for it would require too much code for this book.

pub struct EventReader<'a> {
    atom_urids: &'a AtomURIDCollection,
    midi_urids: &'a MidiURIDCollection,
}

impl<'a> EventReader<'a> {
    pub fn new(atom_urids: &'a AtomURIDCollection, midi_urids: &'a MidiURIDCollection) -> Self {
        Self {
            atom_urids,
            midi_urids,
        }
    }
}

impl<'a> Pipe for EventReader<'a> {
    type InputItem = Option<UnidentifiedAtom<'a>>;
    type OutputItem = (Option<NoteUpdate>, Option<NoteUpdate>);

    fn next(&mut self, atom: Option<UnidentifiedAtom>) -> (Option<NoteUpdate>, Option<NoteUpdate>) {
        if let Some(atom) = atom {
            if let Some(message) = atom
                .read(self.midi_urids.wmidi, ())
            {
                match message {
                    wmidi::MidiMessage::NoteOn(_channel, note, velocity) => {
                        println!("NoteOn: {:?}", note);
                        (Some(NoteUpdate {note: note, onoff: true, velocity: velocity}), Some(NoteUpdate {note: note, onoff: true, velocity: velocity}))
                    }
                    wmidi::MidiMessage::NoteOff(_channel, note, velocity) => {
                        println!("NoteOff: {:?}", note);
                        (Some(NoteUpdate {note: note, onoff: false, velocity: velocity}), Some(NoteUpdate {note: note, onoff: false, velocity: velocity}))
                    }
                    m => {println!("{:?}", m); (None, None)},
                }
            } else {
                println!("could not read atom");
                (None, None)
            }
        } else { (None, None) }
    }
}
