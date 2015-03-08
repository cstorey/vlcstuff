#![feature(unsafe_destructor)]
mod vlc;
use std::ptr;
use std::ffi::{CString,CStr};
use std::old_io::timer::sleep;
use std::time::duration::Duration;
use std::os;
use std::fmt;
use std::sync::{Arc, Mutex};
extern crate libc;

#[derive(Debug)]
pub enum Foo {
        A(*mut vlc::libvlc_media_player_t)
}

#[derive(Debug)]
pub struct VLC {
        inst: *mut vlc::libvlc_instance_t,
}

#[derive(Debug)]
pub struct Player {
        mp: *mut vlc::libvlc_media_player_t,
}

#[derive(Debug)]
pub struct Media {
        item: *mut vlc::libvlc_media_t,
}


impl VLC {
        pub fn new() -> Option<VLC> {
                unsafe {
                let inst = vlc::libvlc_new(0, ptr::null());
                // let mp = vlc::libvlc_media_player_new(inst);
                Some(VLC { inst: inst })
                }
        }
        pub fn new_player(&mut self) -> Player {
                let mp = unsafe { vlc::libvlc_media_player_new(self.inst) };
                Player { mp: mp }
        }

        pub fn open_media(&mut self, s: &str) -> Media {
                let sc = CString::new(s).unwrap();
                let m = unsafe { vlc::libvlc_media_new_location (self.inst, sc.as_ptr()) };
                Media { item: m }
        }
}

impl Drop for VLC {
        fn drop(&mut self) {
                unsafe { vlc::libvlc_release(self.inst) }
        }
}

pub struct EventScope<F>  where F : Fn() {
        handler: F,
        ev: *mut vlc::libvlc_event_manager_t,
        evid: i32,
}

impl <F> EventScope<F> where F : Fn() {
        pub fn new(player: &Player, evid: vlc::libvlc_event_type_t, f: F) -> Box<EventScope<F>>  {
                unsafe {
                        let ev = vlc::libvlc_media_player_event_manager(player.mp);
                        let mut scope = Box::new(EventScope { ev: ev, evid: evid, handler: f });
                        vlc::libvlc_event_attach(ev, evid,  Some(player_ev_cb::<F>), scope.as_voidp());
                        scope
                }
        }

        fn as_voidp(&self) -> *mut libc::c_void {
                &*self as *const _ as *mut libc::c_void
        }
}

impl<F> fmt::Debug for EventScope<F> where F : Fn() {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ EventScope at {:p}; as voidp: {:?} }}", &*self, self.as_voidp())
    }
}

#[unsafe_destructor]
impl<F> Drop for EventScope<F> where F : Fn() {
        fn drop(&mut self) {
                println!("Dropping: {:?}", &self);
                unsafe { vlc::libvlc_event_detach(self.ev, self.evid, Some(player_ev_cb::<F>), self.as_voidp()); }
        }
}
extern fn player_ev_cb<F>(ev: *const vlc::libvlc_event_t,
                      arg2: *mut libc::c_void) where F : Fn() {
        unsafe {
                let thunk: &EventScope<F> = &mut *(arg2 as *mut EventScope<F>);
                (thunk.handler)();
        }
}

#[derive(Debug)]
pub enum PlayerState {
        NothingSpecial,
        Opening,
        Buffering,
        Playing,
        Paused,
        Stopped,
        Ended ,
        Error,
}

impl PlayerState {
        pub fn from_native(state: vlc::libvlc_state_t) -> Option<PlayerState> {
                match state {
                vlc::libvlc_NothingSpecial => Some(PlayerState::NothingSpecial),
                vlc::libvlc_Opening  => Some(PlayerState::Opening),
                vlc::libvlc_Buffering  => Some(PlayerState::Buffering),
                vlc::libvlc_Playing  => Some(PlayerState::Playing),
                vlc::libvlc_Paused  => Some(PlayerState::Paused),
                vlc::libvlc_Stopped  => Some(PlayerState::Stopped),
                vlc::libvlc_Ended  => Some(PlayerState::Ended),
                vlc::libvlc_Error  => Some(PlayerState::Error),
                default => None
                }
        }
}

impl Player {
        pub fn set_media(&mut self, m: &Media) {
                unsafe { vlc::libvlc_media_player_set_media(self.mp, m.item) }
        }
        pub fn get_position(&mut self) -> f32 {
                unsafe { vlc::libvlc_media_player_get_position(self.mp) }
        }

        pub fn get_media(&mut self) -> Media {
                let m = unsafe { vlc::libvlc_media_player_get_media(self.mp) };
                Media { item: m }
        }

        pub fn get_state(&mut self) -> Option<PlayerState> {
                let state = unsafe { vlc::libvlc_media_player_get_state(self.mp) };
                PlayerState::from_native(state)
        }

        pub fn play(&mut self) {
                let ret = unsafe { vlc::libvlc_media_player_play (self.mp) };
                println!("Play: {}", ret);
        }
        pub fn stop(&mut self) {
                unsafe { vlc::libvlc_media_player_stop (self.mp) };
        }

        pub fn on_position_changed<F>(&mut self, f: F) -> Box<EventScope<F>> where F : Fn() {
                let evid : vlc::libvlc_event_type_t = vlc::libvlc_MediaPlayerPositionChanged as vlc::libvlc_event_type_t;
                EventScope::new(self, evid, f)
        }

        pub fn on_media_end_reached<F>(&mut self, f: F) -> Box<EventScope<F>> where F : Fn() {
                let evid : vlc::libvlc_event_type_t = vlc::libvlc_MediaPlayerEndReached as vlc::libvlc_event_type_t;
                EventScope::new(self, evid, f)
        }
}

unsafe impl Send for Player {}

impl Drop for Player {
        fn drop(&mut self) {
                unsafe {
                        vlc::libvlc_media_player_release (self.mp);
                }
        }
}

impl Media {
        pub fn get_duration(&self) -> Duration {
                let duration_ms = unsafe { vlc::libvlc_media_get_duration(self.item) };
                Duration::milliseconds(duration_ms)
        }
}


impl Drop for Media {
        fn drop(&mut self) {
                unsafe {
                        vlc::libvlc_media_release (self.item)
                }
        }
}


pub fn main() {
        let mut inst = VLC::new().unwrap();
        let args = os::args();
        let mut a : Vec<_> = args.iter().skip(1).collect();
        let s = a.pop().unwrap().as_slice();
        let m = inst.open_media(s);
        let mpp = Arc::new(Mutex::new(inst.new_player()));


        let cb = || {
                let ref mut player = mpp.lock().unwrap();
                let frac_base : i32 = 1<<16;
                let pos_frac = player.get_position();
                let pos : i32 = (pos_frac * frac_base as f32) as i32;
                let dur = player.get_media().get_duration();
                let state = player.get_state();
                println!("State: {:?}; Pos: {}/{}", state, dur*pos/frac_base, dur)
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
