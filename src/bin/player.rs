#![feature(old_io, std_misc, core)]
#![allow(unused_assignments, unused_variables)] // For the scoped event guards below.
extern crate vlcstuff as vlc;
use std::old_io::timer::sleep;
use std::time::duration::Duration;
use std::env;
use std::sync::{Arc, Mutex};
use vlc::*;

pub fn main() {
        let mut inst = VLC::new().unwrap();
        let mut a : Vec<_> = env::args().skip(1).collect();
        let s = a.pop().unwrap();
        let m = inst.open_media(s.as_slice());
        let mpp = Arc::new(Mutex::new(inst.new_player()));


        let cb = || {
                let ref mut player = mpp.lock().unwrap();
                let pos_frac = player.get_position();
                let dur = player.get_media().get_duration();
                let pos = Duration::milliseconds((pos_frac * dur.num_milliseconds() as f32) as i64);
                let state = player.get_state();
                println!("State: {:?}; Pos: {}/{}", state, pos, dur)
        };

        let pos_event_guard;
        let ended_event_guard;
        {
                let ref mut mp = mpp.lock().unwrap();
                pos_event_guard = mp.on_position_changed(cb);
                ended_event_guard = mp.on_media_end_reached(|| {
                        let ref mut player = mpp.lock().unwrap();
                        println!("Ended: {:?}", player.get_state());
                });
                mp.set_media(&m);
                mp.play();
        }
        sleep (Duration::seconds(10)); /* Let it play a bit */
        mpp.lock().unwrap().stop();
}
