#![feature(old_io, std_misc, core)]
#![allow(unused_assignments, unused_variables)] // For the scoped event guards below.
extern crate vlcstuff as vlc;
use std::old_io::timer::sleep;
use std::time::duration::Duration;
use std::env;
use std::path::Path;
use vlc::*;

pub fn main() {
        let mut inst = VLC::new().unwrap();
        let mut a : Vec<_> = env::args().skip(1).collect();
    for s in a {
        let m = inst.open_path(Path::new(s.as_slice()));
        println!("{}: {}", s, m.get_trackname());
    }
}
