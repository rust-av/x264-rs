#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod x264 {
    include!(concat!(env!("OUT_DIR"), "/x264.rs"));
}

pub use x264::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    #[test]
    fn init_and_version() {
        unsafe {
            let mut par = mem::MaybeUninit::uninit();
            x264_param_default(par.as_mut_ptr());
            let mut par = par.assume_init();
            par.i_width = 640;
            par.i_height = 480;
            let x = x264_encoder_open(&mut par);

            x264_encoder_close(x);
        }
    }
}
