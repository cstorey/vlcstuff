#![feature(unsafe_destructor)]
extern crate "vlcstuff" as vlc;
use std::ptr;
use std::ffi::{CString,CStr};
use std::old_io::timer::sleep;
use std::time::duration::Duration;
use std::os;
use std::fmt;
use std::sync::{Arc, Mutex};
use vlc::*;

pub fn main() {
        let mut inst = VLC::new().unwrap();
        let args = os::args();
        let mut a : Vec<_> = args.iter().skip(1).collect();
        let s = a.pop().unwrap().as_slice();
        let m = inst.open_media(s);
        let mpp = Arc::new(Mutex::new(inst.new_player()));


        let cb = || {
                let ref mut player = mpp.lock().unwrap();
                let pos_frac = player.get_position();
                let dur = player.get_media().get_duration();
                let pos = Duration::milliseconds((pos_frac * dur.num_milliseconds() as f32) as i64);
                let state = player.get_state();
                println!("State: {:?}; Pos: {}/{}", state, pos, dur)
        };

        let pos;
        let ended;
        {
                let ref mut mp = mpp.lock().unwrap();
                pos = mp.on_position_changed(cb);
                ended = mp.on_media_end_reached(|| {
                        let ref mut player = mpp.lock().unwrap();
                        println!("Ended: {:?}", player.get_state());
                });
                mp.set_media(&m);
                mp.play();
        }
        sleep (Duration::seconds(10)); /* Let it play a bit */
        mpp.lock().unwrap().stop();
}
