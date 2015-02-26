mod vlc;
use std::ptr;
use std::ffi::{CString,CStr};
use std::old_io::timer::sleep;
use std::time::duration::Duration;
use std::os;
extern crate libc;

#[derive(Debug)]
pub enum Foo {
        A(*mut vlc::libvlc_media_player_t)
}

pub struct VLC {
        pub inst: *mut vlc::libvlc_instance_t,
        pub mp: *mut vlc::libvlc_media_player_t,
}

impl VLC {
        pub fn new() -> Option<VLC> {
                unsafe {
                let inst = vlc::libvlc_new(0, ptr::null());
                let mp = vlc::libvlc_media_player_new(inst);
                Some(VLC { inst: inst, mp: mp })
                }
        }

        pub fn open(&mut self, s: &str) {
                unsafe {
                let sc = CString::new(s).unwrap();
                let m = vlc::libvlc_media_new_location (self.inst, sc.as_ptr());
                vlc::libvlc_media_player_set_media(self.mp, m);
                vlc::libvlc_media_release (m)
                }
        }
}

impl Drop for VLC {
        fn drop(&mut self) {
                unsafe {
                        vlc::libvlc_media_player_release (self.mp);
                        vlc::libvlc_release(self.inst)
                }
        }
}

extern fn ev_callback(ev: *const vlc::libvlc_event_t,
                      arg2: *mut libc::c_void) {
        unsafe {
        let data: &mut Foo = &mut *(arg2 as *mut Foo);
        let Foo::A(mp) = *data;
        let e = unsafe { *ev };
        let frac_pos = vlc::libvlc_media_player_get_position(mp);
        let m = vlc::libvlc_media_player_get_media(mp);
        let duration = vlc::libvlc_media_get_duration(m) as f32 / 1000.0;
        println!("Pos: {}/{} {}s", frac_pos, duration, duration*frac_pos)
        }
}

pub fn xmain() {
        let mut vlc = VLC::new().unwrap();
        let args = os::args();
        let mut a : Vec<_> = args.iter().skip(1).collect();
        let s = a.pop().unwrap().as_slice();
        vlc.open(s)
}

pub fn main() {
unsafe {
        let inst = vlc::libvlc_new(0, ptr::null());
//     /* Create a new item */
        let args = os::args();
        let mut a : Vec<_> = args.iter().skip(1).collect();
        let s = a.pop().unwrap().as_slice();
        let sc = CString::new(s).unwrap();
        let m = vlc::libvlc_media_new_location (inst, sc.as_ptr());

     /* Create a media player playing environement */
        let mp = vlc::libvlc_media_player_new_from_media (m);
        let ev = vlc::libvlc_media_player_event_manager(mp);
        let mut dat = Foo::A(mp);
        let mut datp : *mut libc::c_void = &mut dat as *mut _ as *mut libc::c_void;
        let evid : vlc::libvlc_event_type_t = vlc::libvlc_MediaPlayerPositionChanged as vlc::libvlc_event_type_t;
        vlc::libvlc_event_attach(ev, evid,  Some(ev_callback), datp);

     /* No need to keep the media now */
        vlc::libvlc_media_release (m);

     /* play the media_player */
        let ret = vlc::libvlc_media_player_play (mp);
        println!("Play: {}", ret);
//
        sleep (Duration::seconds(10)); /* Let it play a bit */
//
//     /* Stop playing */
//     libvlc_media_player_stop (mp);
//
//     /* Free the media_player */
        vlc::libvlc_media_player_release (mp);
//
        vlc::libvlc_release(inst)
}
}
