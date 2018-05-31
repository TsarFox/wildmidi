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

extern crate ao_rs as ao;
extern crate wildmidi;

use std::env::args;
use std::mem::transmute;
use std::path::Path;
use std::process::exit;

use ao::{Ao, Device, Driver, Format};
use wildmidi::Player;

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        println!("usage: {} [MIDI]", args[0]);
        exit(1);
    }

    let path = &args[1];

    if !Path::new(&path).exists() {
        println!("{} does not exist.", path);
        exit(1);
    }

    let player = Player::new(44100).unwrap();
    let mut midi = player.load_file(&path).unwrap();

    let _ao = Ao::new();
    let driver = Driver::new().unwrap();
    let format = Format::new();
    let device = Device::new(&driver, &format, None).unwrap();

    loop {
        // It would simply be too slow to do a safe conversion every time we
        // buffer the PCM output.
        let vec = midi.play(4096);
        let pcm = unsafe {
            transmute::<&[u8], &[i8]>(&vec[..])
        };

        if pcm.len() <= 0 {
            break;
        }

        device.play(&pcm[..] as &[i8]);
    }
}
