extern crate pkg_config;

fn main() {
        match pkg_config::Config::new().atleast_version("2.0.0").find("libvlc") {
        Ok(_) => return,
        Err(..) => panic!("Need to have libvlc with dev packages / pkg-config installed")
    }
}
