#![feature(unsafe_destructor)]
#![feature(libc)]
#![feature(convert)]
extern crate libvlc_sys as vlc;
extern crate time;
use std::ptr;
use std::ffi::{CString,CStr};
use std::str;
use time::Duration;
use std::os;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::path::Path;
extern crate libc;

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

        pub fn open_location(&mut self, s: &str) -> Media {
                let sc = CString::new(s).unwrap();
                let m = unsafe { vlc::libvlc_media_new_location (self.inst, sc.as_ptr()) };
                Media { item: m }
        }
        pub fn open_path(&mut self, p: &Path) -> Media {
                let sc = p.as_os_str().to_cstring().unwrap();
                let m = unsafe { vlc::libvlc_media_new_path (self.inst, sc.as_ptr()) };
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
        write!(f, "{{ EventScope at {:p}; }}", &*self)
    }
}

#[unsafe_destructor]
impl<F> Drop for EventScope<F> where F : Fn() {
        fn drop(&mut self) {
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
    fn _parse_meta(&self) {
        unsafe { vlc::libvlc_media_parse(self.item) }
    }
    pub fn get_trackname(&self) -> &str {
        self._parse_meta();
        let s = unsafe { CStr::from_ptr(vlc::libvlc_media_get_meta(self.item, vlc::libvlc_meta_Title)) };
        str::from_utf8(s.to_bytes()).unwrap()
    }
}


impl Drop for Media {
        fn drop(&mut self) {
                unsafe {
                        vlc::libvlc_media_release (self.item)
                }
        }
}
