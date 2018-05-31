This is a simple Rust wrapper around the WildMIDI software synthesizer library.


# Examples

Included in this repository is a binary project exposing a very basic MIDI
player - essentially a clone of wildmidi(1) that is missing a majority of the
features.

```
$ cd examples/player/
$ cargo run D_E1A1.MID
```


# Tests

The WildMIDI C library maintains some global state, so tests should **not** be
run in parallel. Running tests consecutively can be achieved by invoking
cargo-test as:

```
cargo test -- --test-threads=1
```


# TODO

- [x] Implement wrappers for WildMidi_GetInfo.
- [ ] Implement wrappers for WildMidi_SetOption.
- [ ] Valid MIDI blob for use in testing 'load' and the methods of 'Midi'.
- [ ] Compile the WildMIDI C library, or provide more sanity checking to ensure that it's present?