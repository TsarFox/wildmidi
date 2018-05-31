// Copyright (C) 2018 Jakob L. Kreuze, All Rights Reserved.
//
// This file is part of wildmidi.
//
// wildmidi is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// wildmidi is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
// details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with wildmidi. If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate simple_error;

use std::error::Error;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_uchar, c_ushort, c_int, c_ulong, c_void};
use std::path::Path;

#[repr(C)]
struct WM_Info {
    copyright: *const c_char,
    current_sample: c_ulong,
    approx_total_samples: c_ulong,
    total_midi_time: c_ulong,
    mixer_options: c_ushort,
}

extern "C" {
    // "Methods" of the Midi player.
    fn WildMidi_Init(cfg: *const c_char, rate: c_ushort, flags: c_ushort) -> c_int;
    fn WildMidi_Open(path: *const c_char) -> *const c_void;
    fn WildMidi_OpenBuffer(data: *const c_uchar, size: c_ulong) -> *const c_void;
    // fn WildMidi_SetOption();
    fn WildMidi_MasterVolume(volume: c_uchar) -> c_int;
    fn WildMidi_Shutdown();

    // "Methods" of the individual Midi handles.
    fn WildMidi_Close(ptr: *const c_void) -> c_int;
    fn WildMidi_FastSeek(ptr: *const c_void, pos: c_ushort) -> c_int;
    fn WildMidi_GetOutput(ptr: *const c_void, buf: *mut c_uchar, len: c_ulong) -> c_int;
    fn WildMidi_GetInfo(ptr: *const c_void) -> *const WM_Info;
}

/// Loader for the Midi format.
pub struct Player;

impl Player {
    fn locate_cfg() -> Option<&'static str> {
        let paths = vec![
            "/etc/wildmidi/wildmidi.cfg",
            "/etc/wildmidi.cfg"
        ];

        for path in paths.iter() {
            if Path::new(path).exists() {
                return Some(path);
            }
        }

        None
    }

    /// Create a new Player with the given sample rate, using the default
    /// configuration file.
    ///
    /// # Errors
    ///
    /// Will fail if 'rate' is not on the interval [11025,65535], or if neither
    /// of the default configuration files exist ('/etc/wildmidi/wildmidi.cfg',
    /// '/etc/wildmidi.cfg').
    pub fn new(rate: u16) -> Result<Player, Box<Error>> {
        let cfg = match Player::locate_cfg() {
            Some(cfg) => cfg,
            None => bail!("No valid configuration file found"),
        };

        Player::with_cfg(cfg, rate)
    }

    /// Create a new Player with the given config path and sample rate.
    ///
    /// # Errors
    ///
    /// Will fail if 'rate' is not on the interval [11025,65535].
    pub fn with_cfg(cfg: &str, rate: u16) -> Result<Player, Box<Error>> {
        let cfg = CString::new(cfg)?;

        unsafe {
            // WildMidi_Shutdown();
            if WildMidi_Init(cfg.as_ptr(), rate, 0) != 0 {
                bail!("Couldn't initialize WildMidi.");
            }
        }

        Ok(Player { })
    }

    /// Sets the overall library volume level. The default is 100.
    pub fn volume(&mut self, volume: u8) -> Result<(), Box<Error>> {
        unsafe {
            if WildMidi_MasterVolume(volume) != 0 {
                bail!("Couldn't set volume.");
            }
        }

        Ok(())
    }

    /// Loads a Midi file from memory.
    ///
    /// # Errors
    ///
    /// Will fail if an internal error occurs in WildMidi (such as a parse
    /// error).
    pub fn load(&self, data: &[u8]) -> Result<Midi, Box<Error>> {
        unsafe {
            let len = data.len() as c_ulong;
            let ptr = WildMidi_OpenBuffer(data.as_ptr(), len);

            if !ptr.is_null() {
                return Ok(Midi::new(ptr));
            }
        }

        bail!("Failed to open Midi file.")
    }

    /// Loads a Midi file from disk.
    ///
    /// # Errors
    ///
    /// Will fail if the file does not exist, or if an internal error occurs in
    /// WildMidi (such as a parse error).
    pub fn load_file(&self, path: &str) -> Result<Midi, Box<Error>> {
        if !Path::new(path).exists() {
            bail!("File does not exist");
        }

        let path = CString::new(path)?;

        unsafe {
            let ptr = WildMidi_Open(path.as_ptr());

            if !ptr.is_null() {
                return Ok(Midi::new(ptr));
            }
        }

        bail!("Failed to open Midi file.")
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        unsafe {
            WildMidi_Shutdown();
        }
    }
}

/// An actual Midi file, capable of producing a PCM output.
pub struct Midi {
    ptr: *const c_void,
}

impl Midi {
    fn new(ptr: *const c_void) -> Midi {
        Midi { ptr }
    }

    /// Returns a Vec<u8> containing 'len' bytes of PCM data.
    pub fn play(&mut self, len: usize) -> Vec<u8> {
        let mut vec = vec![0;len];

        unsafe {
            let buf = vec.as_mut_ptr();
            let handle = self.ptr;
            let read = WildMidi_GetOutput(handle, buf, len as c_ulong) as usize;

            if read < len {
                vec.resize(read, 0);
            }
        }

        vec
    }

    /// Resets all note specific midi states and active notes before scanning to
    /// sample_pos samples from the beginning taking note of any changes to midi
    /// channel states.
    pub fn seek(&mut self, pos: u32) {
        unsafe {
            // FIXME: Doesn't check return value.
            WildMidi_FastSeek(self.ptr, pos as c_ushort);
        }
    }

    /// Returns a string containing any copyright MIDI events, if any were
    /// found.
    pub fn copyright(&self) -> Option<String> {
        unsafe {
            let ptr = WildMidi_GetInfo(self.ptr);

            if (*ptr).copyright.is_null() {
                None
            } else {
                if let Ok(str) = CStr::from_ptr((*ptr).copyright).to_str() {
                    Some(String::from(str))
                } else {
                    None
                }
            }
        }
    }

    /// The number of stereo samples the player has processed so far. Dividing
    /// this value by the player's 'rate' determines the current playing time.
    pub fn current_sample(&self) -> usize {
        unsafe {
            let ptr = WildMidi_GetInfo(self.ptr);
            (*ptr).current_sample as usize
        }
    }

    /// The number of stereo samples the player expects to process. Dividing
    /// this value by the player's 'rate' determines the MIDI's total length.
    pub fn total_samples(&self) -> usize {
        unsafe {
            let ptr = WildMidi_GetInfo(self.ptr);
            (*ptr).approx_total_samples as usize
        }
    }

    /// This is the total time of MIDI events in 1/1000's of a second. It
    /// differs from 'total_samples' in that it only states the total time
    /// within the MIDI file, and does not take into account the extra bit of
    /// time to finish playing sampling smoothly.
    pub fn total_time(&self) -> usize {
        unsafe {
            let ptr = WildMidi_GetInfo(self.ptr);
            (*ptr).total_midi_time as usize
        }
    }
}

impl Drop for Midi {
    fn drop(&mut self) {
        unsafe {
            // There isn't much of a point in handling errors here.
            WildMidi_Close(self.ptr);
        }
    }
}

#[cfg(test)]
mod player_tests {
    use ::*;

    #[test]
    fn create() {
        if let Err(e) = Player::new(44100) {
            panic!("{}", e);
        }
    }

    #[test]
    fn invalid_rate() {
        if let Ok(_) = Player::new(0) {
            panic!("Allowed player to be created with invalid rate.");
        }
    }
}
