use lv2::prelude::*;

#[uri("https://github.com/duffrecords/mellotron#PluginConfig")]
pub struct PluginConfig;

#[uri("https://github.com/duffrecords/mellotron#ui_on")]
pub struct UIOn;

#[uri("https://github.com/duffrecords/mellotron#ui_off")]
pub struct UIOff;

#[uri("https://github.com/duffrecords/mellotron#voice_a_prev")]
pub struct VoiceAPrev;

#[uri("https://github.com/duffrecords/mellotron#voice_a_next")]
pub struct VoiceANext;

#[uri("https://github.com/duffrecords/mellotron#voice_b_prev")]
pub struct VoiceBPrev;

#[uri("https://github.com/duffrecords/mellotron#voice_b_next")]
pub struct VoiceBNext;

#[uri("https://github.com/duffrecords/mellotron#PluginState")]
pub struct PluginState;

#[uri("https://github.com/duffrecords/mellotron#AudioData")]
pub struct AudioData;

#[uri("https://github.com/duffrecords/mellotron#MixValue")]
pub struct MixValue;

#[uri("https://github.com/duffrecords/mellotron#AttackPoint")]
pub struct AttackPoint;

#[uri("https://github.com/duffrecords/mellotron#ReleasePoint")]
pub struct ReleasePoint;

#[uri("https://github.com/duffrecords/mellotron#IdlePoint")]
pub struct IdlePoint;

#[uri("https://github.com/duffrecords/mellotron#gain_signal")]
pub struct GainSignal;

#[uri("https://github.com/duffrecords/mellotron#input_signal")]
pub struct InputSignal;

#[uri("https://github.com/duffrecords/mellotron#output_signal")]
pub struct OutputSignal;

#[uri("http://lv2plug.in/ns/ext/patch#Set")]
pub struct PatchSet;

#[uri("http://lv2plug.in/ns/ext/patch#Get")]
pub struct PatchGet;

#[uri("http://lv2plug.in/ns/ext/patch#Put")]
pub struct PatchPut;

#[uri("http://lv2plug.in/ns/ext/patch#body")]
pub struct PatchBody;

#[uri("http://lv2plug.in/ns/ext/patch#property")]
pub struct PatchProperty;

#[uri("http://lv2plug.in/ns/ext/patch#value")]
pub struct PatchValue;

#[derive(URIDCollection)]
pub struct PatchURIDCollection {
    pub set: URID<PatchSet>,
    pub get: URID<PatchGet>,
    pub put: URID<PatchPut>,
    pub body: URID<PatchBody>,
    pub property: URID<PatchProperty>,
    pub value: URID<PatchValue>
}

#[uri("http://lv2plug.in/ns/ext/atom#Path")]
pub struct AtomPath;

impl<'a, 'b> Atom<'a, 'b> for AtomPath
where 'a: 'b,
{
    type ReadParameter = ();
    type ReadHandle = &'a str;

    type WriteParameter = ();
    type WriteHandle = AtomPathWriter<'a, 'b>;

    fn read(body: Space<'a>, _: ()) -> Option<&'a str> {
        body.data()
            .and_then(|data| std::str::from_utf8(data).ok())
            .map(|path| path.trim_matches(char::from(0)))
    }

    fn init(frame: FramedMutSpace<'a, 'b>, _: ()) -> Option<AtomPathWriter<'a, 'b>> {
        Some(AtomPathWriter { frame })
    }
}

pub struct AtomPathWriter<'a, 'b> {
    frame: FramedMutSpace<'a, 'b>
}

impl<'a, 'b> AtomPathWriter<'a, 'b> {
    pub fn append(&mut self, string: &str) -> Option<&mut str> {
        let data = string.as_bytes();
        let space = self.frame.write_raw(data, false)?;
        unsafe { Some(std::str::from_utf8_unchecked_mut(space)) }
    }
}

pub enum Bank {
    A,
    B,
}

#[uri("https://github.com/duffrecords/mellotron#folder")]
pub struct Folder;


#[derive(URIDCollection)]
pub struct URIDs {
    pub atom: AtomURIDCollection,
    pub midi: MidiURIDCollection,
    pub unit: UnitURIDCollection,
    pub patch: PatchURIDCollection,
    pub folder: URID<Folder>,
    pub atom_path: URID<AtomPath>,
    pub buf_size: BufSizeURIDCollection,
    pub parameters: ParametersURIDCollection,
    pub ui: UIURIDCollection,
    pub plugin_config: URID<PluginConfig>,
    pub ui_on: URID<UIOn>,
    pub ui_off: URID<UIOff>,
    pub voice_a_prev: URID<VoiceAPrev>,
    pub voice_a_next: URID<VoiceANext>,
    pub voice_b_prev: URID<VoiceBPrev>,
    pub voice_b_next: URID<VoiceBNext>,
    pub plugin_state: URID<PluginState>,
    pub mix_value: URID<MixValue>,
    pub attack_point: URID<AttackPoint>,
    pub release_point: URID<ReleasePoint>,
    pub idle_point: URID<IdlePoint>,
    pub audio_data: URID<AudioData>,
    pub input_signal: URID<InputSignal>,
    pub output_signal: URID<OutputSignal>,
    pub gain_signal: URID<GainSignal>
}

#[derive(Copy, Clone)]
pub enum PatchName {
    Cello,
    MkIIViolins,
    M300A,
    M300B,
    StringSection,
    Orchestra,
    MkIICombinedBrass,
    MixedBrassB,
    GC3Brass,
    Trombone,
    Trumpet,
    TromboneAndTrumpet,
    TenorSax,
    TenorAndAltoSax,
    MkIIFlute,
    Bassoon,
    Woodwind2,
    ChurchOrgan,
    ChurchPipeOrgan,
    ItalianAccordian,
    Vibes,
    EightChoir,
    CombinedChoir,
}

#[derive(Copy, Clone)]
pub struct Patch {
    pub name: PatchName,
}

impl Patch {
    pub fn new(name: PatchName) -> Self {
        Self {
            name: name,
        }
    }
    pub fn prev(&mut self) {
        self.name = match self.name {
            PatchName::Cello => PatchName::CombinedChoir,
            PatchName::MkIIViolins => PatchName::Cello,
            PatchName::M300A => PatchName::MkIIViolins,
            PatchName::M300B => PatchName::M300A,
            PatchName::StringSection => PatchName::M300B,
            PatchName::Orchestra => PatchName::StringSection,
            PatchName::MkIICombinedBrass => PatchName::Orchestra,
            PatchName::MixedBrassB => PatchName::MkIICombinedBrass,
            PatchName::GC3Brass => PatchName::MixedBrassB,
            PatchName::Trombone => PatchName::GC3Brass,
            PatchName::Trumpet => PatchName::Trombone,
            PatchName::TromboneAndTrumpet => PatchName::Trumpet,
            PatchName::TenorSax => PatchName::TromboneAndTrumpet,
            PatchName::TenorAndAltoSax => PatchName::TenorSax,
            PatchName::MkIIFlute => PatchName::TenorAndAltoSax,
            PatchName::Bassoon => PatchName::MkIIFlute,
            PatchName::Woodwind2 => PatchName::Bassoon,
            PatchName::ChurchOrgan => PatchName::Woodwind2,
            PatchName::ChurchPipeOrgan => PatchName::ChurchOrgan,
            PatchName::ItalianAccordian => PatchName::ChurchPipeOrgan,
            PatchName::Vibes => PatchName::ItalianAccordian,
            PatchName::EightChoir => PatchName::Vibes,
            PatchName::CombinedChoir => PatchName::EightChoir,
        }
    }
    pub fn next(&mut self) {
        self.name = match self.name {
            PatchName::Cello => PatchName::MkIIViolins,
            PatchName::MkIIViolins => PatchName::M300A,
            PatchName::M300A => PatchName::M300B,
            PatchName::M300B => PatchName::StringSection,
            PatchName::StringSection => PatchName::Orchestra,
            PatchName::Orchestra => PatchName::MkIICombinedBrass,
            PatchName::MkIICombinedBrass => PatchName::MixedBrassB,
            PatchName::MixedBrassB => PatchName::GC3Brass,
            PatchName::GC3Brass => PatchName::Trombone,
            PatchName::Trombone => PatchName::Trumpet,
            PatchName::Trumpet => PatchName::TromboneAndTrumpet,
            PatchName::TromboneAndTrumpet => PatchName::TenorSax,
            PatchName::TenorSax => PatchName::TenorAndAltoSax,
            PatchName::TenorAndAltoSax => PatchName::MkIIFlute,
            PatchName::MkIIFlute => PatchName::Bassoon,
            PatchName::Bassoon => PatchName::Woodwind2,
            PatchName::Woodwind2 => PatchName::ChurchOrgan,
            PatchName::ChurchOrgan => PatchName::ChurchPipeOrgan,
            PatchName::ChurchPipeOrgan => PatchName::ItalianAccordian,
            PatchName::ItalianAccordian => PatchName::Vibes,
            PatchName::Vibes => PatchName::EightChoir,
            PatchName::EightChoir => PatchName::CombinedChoir,
            PatchName::CombinedChoir => PatchName::Cello,
        }
    }
    pub fn to_string(&mut self) -> std::string::String {
        match self.name {
            PatchName::Cello => std::string::String::from("Cello"),
            PatchName::MkIIViolins => std::string::String::from("MkII Violins"),
            PatchName::M300A => std::string::String::from("M300A (Two Violins)"),
            PatchName::M300B => std::string::String::from("M300B (Solo Violin)"),
            PatchName::StringSection => std::string::String::from("String Section"),
            PatchName::Orchestra => std::string::String::from("Orchestra"),
            PatchName::MkIICombinedBrass => std::string::String::from("MkII Combined Brass"),
            PatchName::MixedBrassB => std::string::String::from("Mixed Brass B"),
            PatchName::GC3Brass => std::string::String::from("GC3 Brass"),
            PatchName::Trombone => std::string::String::from("Trombone"),
            PatchName::Trumpet => std::string::String::from("Trumpet"),
            PatchName::TromboneAndTrumpet => std::string::String::from("Trombone and Trumpet"),
            PatchName::TenorSax => std::string::String::from("Tenor Sax"),
            PatchName::TenorAndAltoSax => std::string::String::from("Tenor and Alto Sax"),
            PatchName::MkIIFlute => std::string::String::from("MkII Flute"),
            PatchName::Bassoon => std::string::String::from("Bassoon"),
            PatchName::Woodwind2 => std::string::String::from("Woodwind 2"),
            PatchName::ChurchOrgan => std::string::String::from("Church Organ (Lowry)"),
            PatchName::ChurchPipeOrgan => std::string::String::from("Church Pipe Organ"),
            PatchName::ItalianAccordian => std::string::String::from("Italian Accordian"),
            PatchName::Vibes => std::string::String::from("Vibes"),
            PatchName::EightChoir => std::string::String::from("8 Choir"),
            PatchName::CombinedChoir => std::string::String::from("Combined Choir"),
        }
    }
    pub fn to_fs_string(&mut self) -> std::string::String {
        self.to_string().to_lowercase().replace(" ", "_").replace("(", "").replace(")", "")
    }
}
