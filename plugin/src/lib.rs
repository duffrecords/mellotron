use std::f32::consts::PI;

extern crate lv2;
extern crate lv2_worker;

// use iterpipes::*;
use lv2::prelude::*;
use lv2::lv2_atom as atom;
mod sampler;
use sampler::Sampler;

// mod pipes;
// use pipes::*;

// const VOICE_A: &str = "8 Choir";
// const VOICE_A: &str = "MKII Combined Brass";
const VOICE_A: &str = "MKII Flute";
// const VOICE_A: &str = "MKII Violins";
const VOICE_B: &str = "String Section";
// const VOICE_B: &str = "Combined Choir";

// #[derive(URIDCollection)]
// struct URIDs {
//     atom: AtomURIDCollection,
//     midi: MidiURIDCollection,
//     unit: UnitURIDCollection,
// }

#[derive(PortCollection)]
pub struct Ports {
    enabled: InputPort<Control>,
    // input: InputPort<AtomPort>,
    output: OutputPort<Audio>,
    mix: InputPort<Control>,
    control: InputPort<AtomPort>,
    notify: OutputPort<AtomPort>,
    gain: InputPort<Control>,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

#[derive(FeatureCollection)]
struct AudioFeatures<'a> {
    schedule: lv2_worker::Schedule<'a, Mellotron>,
}


// This plugin struct contains the URID collection and two pre-constructed pipes. These are later used to construct the complete pipeline.
#[uri("https://github.com/duffrecords/mellotron")]
struct Mellotron {
    urids: urids::URIDs,
    ui_active: bool,
    ui_notified: bool,
    // voice_a: Voice,
    // voice_b: Voice,
    sampler: Sampler,
    new_sampler: Option<Sampler>,
    samplerate: f64,
    max_block_length: usize,
    samples_path: Option<std::string::String>,
    state_notification_needed: bool,
    current_gain: f32
}

impl Plugin for Mellotron {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = AudioFeatures<'static>;

    fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        // let voice_a = Voice::new(urids::Patch::new(urids::PatchName::MkIIFlute));
        // let voice_b = Voice::new(urids::Patch::new(urids::PatchName::StringSection));

        let samplerate = plugin_info.sample_rate();
        let max_block_length = 8192; /*FIXME*/

        Some(Self {
            urids: features.map.populate_collection()?,
            ui_active: false,
            ui_notified: false,
            // voice_a,
            // voice_b,
            sampler: Sampler::new(urids::Patch::new(urids::PatchName::MkIIFlute), urids::Patch::new(urids::PatchName::StringSection)).unwrap(),
            // sampler_a: Sampler::new(urids::Patch::new(urids::PatchName::MkIIFlute)).unwrap(),
            // sampler_b: Sampler::new(urids::Patch::new(urids::PatchName::StringSection)).unwrap(),
            new_sampler: None,
            // new_sampler_a: None,
            // new_sampler_b: None,
            samplerate,
            max_block_length,
            samples_path: None,
            state_notification_needed: false,
            current_gain: dB_to_gain(-6.0),
        })
    }

    // fn activate(&mut self, _: &mut Features<'static>) {
    //     self.voice_a.reset();
    //     self.voice_b.reset();
    // }

    fn run(&mut self, ports: &mut Ports, features: &mut Self::AudioFeatures, _: u32) {
        let mut offset: usize = 0;
        self.check_notification_events(ports);
        let mix = *(ports.mix);

        for o in ports.output.iter_mut() {
            *o = 0.0;
        }

        let active_sampler = if let Some(new_sampler) = &mut self.new_sampler {
            if self.sampler.fadeout_finished() {
                self.sampler = self.new_sampler.take().unwrap();
                &mut self.sampler
            } else {
                self.sampler.process(&mut ports.output, mix);
                new_sampler
            }
        } else {
            &mut self.sampler
        };

        let control_sequence = ports
        .control
        .read(self.urids.atom.sequence, self.urids.unit.beat)
        .unwrap();

        for (timestamp, message) in control_sequence {
            match timestamp.as_frames() {
                Some(ts) if ts > 0  => {
                    let frame = ts as usize;
                    println!("DEBUG offset: {:?}\tframe: {:?}", offset, frame);
                    active_sampler.process(&mut ports.output[offset..frame], mix);
                    offset = frame;
                }
                _ => {}
            };

            if let Some(msg) = message.read(self.urids.midi.wmidi, ()) {
                active_sampler.midi_event(&msg);
            };

            if let Some((header, mut object_reader)) = message.read(self.urids.atom.object, ()) {
                println!("DEBUG received message");
                if header.otype == self.urids.patch.set {
                    if let Some(path) = parse_folder_path(&self.urids, &mut object_reader) {
                        if let Err(e) = features.schedule.schedule_work(SamplerParameters {
                            bank: urids::Bank::A,
                            path: path.to_string(),
                            // sfzfile: path.to_string(),
                            // host_samplerate: self.samplerate,
                            // max_block_length: self.max_block_length
                        }) {
                            println!("DEBUG can't schedule work {}", e);
                        } else {
                            println!("DEBUG work scheduled");
                        }
                        self.samples_path = Some(path.to_string());
                    }
                } else if header.otype == self.urids.patch.get {
                    println!("DEBUG recieved get request");
                    self.state_notification_needed = true;
                }
            }
        }

        let nsamples = ports.output.len();
        if offset < nsamples {
            active_sampler.process(&mut ports.output[offset..nsamples], mix);
        }

        let gain_target = match *ports.gain {
            g if g < -80.0 => 0.0,
            g if g >= 20.0 => dB_to_gain(20.0),
            g => dB_to_gain(g)
        };

        let tau = 1.0 - (-2.0 * PI * 25.0 / self.samplerate as f32).exp();
        let mut current_gain = self.current_gain;

        if (tau * (current_gain - gain_target)).abs() < std::f32::EPSILON * current_gain {
            current_gain = gain_target;
        }
        self.current_gain = current_gain;

        for o in ports.output.iter_mut() {
            current_gain += tau * (gain_target - current_gain);
            *o *= current_gain;
        }

        if self.state_notification_needed {//&& self.sfzfile_path.is_some() {
            println!("DEBUG trying to notify");

            let mut object_writer = ports.notify.init(
                self.urids.atom.object,
                ObjectHeader {
                    id: None,
                    otype: self.urids.patch.set.into_general(),
                }
            ).unwrap();

            object_writer.init(self.urids.patch.property,
                               self.urids.atom.urid,
                               self.urids.folder.into_general());

            let mut prop_writer = object_writer.init(self.urids.patch.value,
                                                 self.urids.atom_path, ()).unwrap();
            let test_string = prop_writer.append(self.samples_path.as_ref().unwrap());

            println!("wrote {:?}", test_string);

            self.state_notification_needed = false;
        }

        /*
        // Get the reading handle of the input sequence.
        if let Some(input) = ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
        {
            let input_sequence =
                input.map(|(timestamp, event)| (timestamp.as_frames().unwrap() as usize, event));

            // read incoming events and send a copy of the note on/off data to each voice
            let mut pipeline = EventAtomizer::new(input_sequence).compose()
                >> EventReader::new(&self.urids.atom, &self.urids.midi, &self.urids, &mut ports.notify)
                >> (&mut self.voice_a, &mut self.voice_b);

            // mix voices together and generate a frame for every frame in the output buffer
            for frame in ports.output.iter_mut() {
                let pipe_out = pipeline.next(());
                // *frame = (pipeline.next(()).0 * mix_a + pipeline.next(()).1 * mix_b) * 0.0005;
                // *frame = pipeline.next(()).0 * mix_a * 0.0005;
                *frame = (pipe_out.0 * mix_a + pipe_out.1 * mix_b) * 0.0005;
            }
        }
        */
    }
}

impl Mellotron {
    fn check_notification_events(&mut self, ports: &mut Ports) {
        let control_sequence = match ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat) {
                None => return,
                Some(cs) => cs
            };

        for (_, message) in control_sequence {
            if let Some((header,  _)) = message.read(self.urids.atom.object, ()) {
                if header.otype == self.urids.ui_on {
                    self.ui_active = true;
                    self.ui_notified = false;
                } else if header.otype == self.urids.ui_off {
                    self.ui_active = false;
                } else if header.otype == self.urids.voice_a_prev {
                    self.sampler.a.prev_patch();
                } else if header.otype == self.urids.voice_a_next {
                    self.sampler.a.next_patch();
                } else if header.otype == self.urids.voice_b_prev {
                    self.sampler.b.prev_patch();
                } else if header.otype == self.urids.voice_b_next {
                    self.sampler.b.prev_patch();
                }
            }
        }
    }
}

struct SamplerParameters {
    bank: urids::Bank,
    path: std::string::String,
}

fn parse_folder_path<'a>(urids: &urids::URIDs, object_reader:
                          &mut atom::object::ObjectReader<'a>) -> Option<&'a str> {
    if let Some((property_header, atom)) = object_reader.next() {
        if property_header.key != urids.patch.property {
            return None;
        }
        if atom.read(urids.atom.urid, ()).map_or(true, |urid| urid != urids.folder) {
            return None;
        }
        if let Some((property_header, atom)) = object_reader.next() {
            if property_header.key != urids.patch.value {
                return None;
            }
            let patch = if let Some(patch) = atom.read(urids.atom_path, ()) {
                patch
            } else {
                return None;
            };
            return Some(patch);
        }
    }
    None
}

impl lv2_worker::Worker for Mellotron {
    type WorkData = SamplerParameters;

    type ResponseData = Sampler;

    fn work(response_handler: &lv2_worker::ResponseHandler<Self>, data: Self::WorkData)
            -> Result<(), lv2_worker::WorkerError> {
        // println!("work {}", data.sfzfile);
        match data.bank {
            urids::Bank::A => {
                let sampler = Sampler::new(
                    urids::Patch::new(urids::PatchName::MkIIFlute),
                    urids::Patch::new(urids::PatchName::StringSection),
                )
                .map_err(|e| {
                    println!("failed {:?}", e);
                    lv2_worker::WorkerError::Unknown
                })?;
                response_handler.respond(sampler).map_err(|_| lv2_worker::WorkerError::Unknown)
            }
            urids::Bank::B => {
                let sampler = Sampler::new(
                    urids::Patch::new(urids::PatchName::MkIIFlute),
                    urids::Patch::new(urids::PatchName::StringSection),
                )
                .map_err(|e| {
                    println!("failed {:?}", e);
                    lv2_worker::WorkerError::Unknown
                })?;
                response_handler.respond(sampler).map_err(|_| lv2_worker::WorkerError::Unknown)
            }
        }
        // let engine = soundfonts::sfz::engine::Engine::new(data.sfzfile,
        //                                                   data.host_samplerate,
        //                                                   data.max_block_length)
        //     .map_err(|e| {
        //         println!("failed {:?}", e);
        //         lv2_worker::WorkerError::Unknown
        //     })?;

        // response_handler.respond(engine).map_err(|_| lv2_worker::WorkerError::Unknown)
    }

    fn work_response(&mut self, data: Self::ResponseData, _f: &mut Self::AudioFeatures)
                     -> Result<(), lv2_worker::WorkerError> {
        println!("work_response");
        self.sampler.fadeout();
        self.new_sampler = Some(data);
        // self.state_notification_needed = true;

        Ok(())
    }
}

#[allow(non_snake_case)]
pub fn dB_to_gain(dB: f32) -> f32 {
    let ten: f32 = 10.0;
    ten.powf(0.05 * dB)
}

lv2_descriptors!(Mellotron);
