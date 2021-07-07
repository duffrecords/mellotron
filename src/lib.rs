use iterpipes::*;
use lv2::prelude::*;

mod pipes;
use pipes::*;

// const VOICE_A: &str = "8 Choir";
// const VOICE_A: &str = "MKII Combined Brass";
const VOICE_A: &str = "MKII Flute";
// const VOICE_A: &str = "MKII Violins";
const VOICE_B: &str = "String Section";

#[derive(URIDCollection)]
struct URIDs {
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    unit: UnitURIDCollection,
}

#[derive(PortCollection)]
pub struct Ports {
    input: InputPort<AtomPort>,
    output: OutputPort<Audio>,
    mix: InputPort<Control>,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

// This plugin struct contains the URID collection and two pre-constructed pipes. These are later used to construct the complete pipeline.
#[uri("https://github.com/duffrecords/mellotron")]
pub struct Mellotron {
    urids: URIDs,
    voice_a: Voice,
    voice_b: Voice,
}

impl Plugin for Mellotron {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        let voice_a = Voice::new(VOICE_A.to_string());
        let voice_b = Voice::new(VOICE_B.to_string());

        Some(Self {
            urids: features.map.populate_collection()?,
            voice_a,
            voice_b,
        })
    }

    fn activate(&mut self, _: &mut Features<'static>) {
        self.voice_a.reset();
        self.voice_b.reset();
    }

    fn run(&mut self, ports: &mut Ports, _: &mut (), _: u32) {
        let mix_b = *(ports.mix);
        let mix_a = 1.0 - mix_b;
        // Get the reading handle of the input sequence.
        if let Some(input) = ports
            .input
            .read(self.urids.atom.sequence, self.urids.unit.beat)
        {
            let input_sequence =
                input.map(|(timestamp, event)| (timestamp.as_frames().unwrap() as usize, event));

            // read incoming events and send a copy of the note on/off data to each voice
            let mut pipeline = EventAtomizer::new(input_sequence).compose()
                >> EventReader::new(&self.urids.atom, &self.urids.midi)
                >> (&mut self.voice_a, &mut self.voice_b);

            // mix voices together and generate a frame for every frame in the output buffer
            for frame in ports.output.iter_mut() {
                let pipe_out = pipeline.next(());
                // *frame = (pipeline.next(()).0 * mix_a + pipeline.next(()).1 * mix_b) * 0.0005;
                // *frame = pipeline.next(()).0 * mix_a * 0.0005;
                *frame = (pipe_out.0 * mix_a + pipe_out.1 * mix_b) * 0.0005;
            }
        }
    }
}

lv2_descriptors!(Mellotron);
