# mellotron

This is a polyphonic software instrument in LV2 format, designed to emulate the classic Mellotron from the 60s.

## Installation
The plugin was created using the Rust lv2 framework and can be built with Cargo.  I used taijiguy's Mellotron samples but didn't include them in this repo because their licensing status is unclear.  In any case, they're not hard to find on the web.  Save each set of files to the corresponding folder within `samples/` directory, as shown below:
```bash
samples
├── mkii_flute
└── mkii_violins
    ├── A#2.wav
    ├── A2.wav
    ├── A#3.wav
    └── A3.wav
```

Then run the following script to build the plugin and copy it to your `~/.lv2/` directory:
```bash
./install.sh
```

## Usage
The plugin can be opened with a plugin host such as [Carla](https://github.com/falkTX/Carla).  Open the patchbay and connect a MIDI controller or sequencer to the input and connect the outputs to an appropriate audio sink.  There are 35 samples per voice, from A2 to F5.  Keys outside of this range have no effect.  You can load two voices at a time.  Until I write a UI, the voices must be configured at build time as the constants `VOICE_A` and `VOICE_B` in `src/lib.rs`.  There is a mix control for blending the two, allowing custom sounds to be created.  Voice A is selected by default.
