extern crate x264_sys as ffi;

use std::mem;
use ffi::x264::*;
use std::ptr::null;

// TODO: Provide a builder API instead?
pub struct X264Param {
    par: x264_param_t,
}

impl X264Param {
    pub fn new() -> X264Param {
        let mut par = unsafe { mem::uninitialized() };

        unsafe {
            x264_param_default(&mut par as *mut x264_param_t);
        }

        X264Param { par: par }
    }
    pub fn default_preset(tune: Option<&str>,
                          preset: Option<&str>)
                          -> Result<X264Param, &'static str> {
        let mut par = unsafe { mem::uninitialized() };

        match unsafe {
                  x264_param_default_preset(&mut par as *mut x264_param_t,
                                            tune.map_or_else(|| null(),
                                                             |v| v.as_ptr() as *const i8),
                                            preset.map_or_else(|| null(), |v| v.as_ptr()) as
                                            *const i8)
              } {
            -1 => Err("Invalid Argument"),
            0 => Ok(X264Param { par: par }),
            _ => Err("Unexpected"),
        }
    }
    pub fn apply_profile(&mut self, profile: &str) -> Result<X264Param, &'static str> {
        match unsafe { x264_param_apply_profile(&mut self.par, profile.as_ptr() as *const i8) } {
            -1 => Err("Invalid Argument"),
            0 => Ok(self),
            _ => Err("Unexpected"),
        }
    }
    pub fn param_parse(&mut self, name: &str, value: &str) -> Result<X264Param, &'static str> {
        match unsafe {
                  x264_param_parse(&mut self.par,
                                   name.as_ptr() as *const i8,
                                   value.as_ptr() as *const i8)
              } {
            -1 => Err("Invalid Argument"),
            0 => Ok(self),
            _ => Err("Unexpected"),
        }
    }
}

pub struct X264 {
    enc: *mut x264_t,
}

impl X264 {
    pub fn open(mut par: X264Param) -> Result<X264, &'static str> {
        let enc = unsafe { x264_encoder_open(&mut par.par as *mut x264_param_t) };

        if enc.is_null() {
            Err("Out of Memory")
        } else {
            Ok(X264 { enc: enc })
        }
    }
}

impl Drop for X264 {
    fn drop(&mut self) {
        unsafe { x264_encoder_close(self.enc) };

    }
}
