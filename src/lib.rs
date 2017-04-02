extern crate x264_sys as ffi;

use std::mem;
use ffi::x264::*;
use std::ptr::null;
use std::os::raw::c_int;

pub struct Picture {
    pic: x264_picture_t,
    height: i32, // to compute the slice dimension
    native: bool,
}

impl Picture {
    /*
    pub fn new() -> Picture {
        let mut pic = unsafe { mem::uninitialized() };

        unsafe { x264_picture_init(&mut pic as *mut x264_picture_t) };

        Picture { pic: pic }
    }
*/
    pub fn from_param(param: &Param) -> Result<Picture, &'static str> {
        let mut pic = unsafe { mem::uninitialized() };

        let ret = unsafe {
            x264_picture_alloc(&mut pic as *mut x264_picture_t,
                               param.par.i_csp,
                               param.par.i_width,
                               param.par.i_height)
        };
        if ret < 0 {
            Err("Allocation Failure")
        } else {
            Ok(Picture {
                   pic: pic,
                   height: param.par.i_height,
                   native: true,
               })
        }
    }

    pub fn as_slice<'a>(&'a self, plane: usize) -> Result<&'a [u8], &'static str> {
        if plane > self.pic.img.i_plane as usize {
            Err("Invalid Argument")
        } else {
            let size = self.height * self.pic.img.i_stride[plane];
            Ok(unsafe { std::slice::from_raw_parts(self.pic.img.plane[plane], size as usize) })
        }
    }

    pub fn as_slice_mut<'a>(&'a mut self, plane: usize) -> Result<&'a mut [u8], &'static str> {
        if plane > self.pic.img.i_plane as usize {
            Err("Invalid Argument")
        } else {
            let size = self.height * self.pic.img.i_stride[plane];
            Ok(unsafe { std::slice::from_raw_parts_mut(self.pic.img.plane[plane], size as usize) })
        }
    }

    pub fn set_timestamp(mut self, pts: i64) -> Picture {
        self.pic.i_pts = pts;
        self
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        if self.native {
            unsafe { x264_picture_clean(&mut self.pic as *mut x264_picture_t) };
        }
    }
}

// TODO: Provide a builder API instead?
pub struct Param {
    par: x264_param_t,
}

impl Param {
    pub fn new() -> Param {
        let mut par = unsafe { mem::uninitialized() };

        unsafe {
            x264_param_default(&mut par as *mut x264_param_t);
        }

        Param { par: par }
    }
    pub fn default_preset(tune: Option<&str>, preset: Option<&str>) -> Result<Param, &'static str> {
        let mut par = unsafe { mem::uninitialized() };

        match unsafe {
                  x264_param_default_preset(&mut par as *mut x264_param_t,
                                            tune.map_or_else(|| null(),
                                                             |v| v.as_ptr() as *const i8),
                                            preset.map_or_else(|| null(), |v| v.as_ptr()) as
                                            *const i8)
              } {
            -1 => Err("Invalid Argument"),
            0 => Ok(Param { par: par }),
            _ => Err("Unexpected"),
        }
    }
    pub fn apply_profile(mut self, profile: &str) -> Result<Param, &'static str> {
        match unsafe { x264_param_apply_profile(&mut self.par, profile.as_ptr() as *const i8) } {
            -1 => Err("Invalid Argument"),
            0 => Ok(self),
            _ => Err("Unexpected"),
        }
    }
    pub fn param_parse<'a>(mut self, name: &str, value: &str) -> Result<Param, &'static str> {
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
    pub fn set_dimension(mut self, height: usize, width: usize) -> Param {
        self.par.i_height = height as c_int;
        self.par.i_width = width as c_int;

        self
    }
}

// TODO: Expose a NAL abstraction
pub struct NalData {
    vec: Vec<u8>,
}

impl NalData {
    fn with_capacity(capacity: usize) -> NalData {
        NalData { vec: Vec::with_capacity(capacity) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.vec.as_slice()
    }
}

pub struct Encoder {
    enc: *mut x264_t,
}

impl Encoder {
    pub fn open(par: &mut Param) -> Result<Encoder, &'static str> {
        let enc = unsafe { x264_encoder_open(&mut par.par as *mut x264_param_t) };

        if enc.is_null() {
            Err("Out of Memory")
        } else {
            Ok(Encoder { enc: enc })
        }
    }

    pub fn get_headers(&mut self) -> Result<NalData, &'static str> {
        let mut nb_nal: c_int = 0;
        let mut c_nals: *mut x264_nal_t = unsafe { mem::uninitialized() };

        let bytes = unsafe {
            x264_encoder_headers(self.enc,
                                 &mut c_nals as *mut *mut x264_nal_t,
                                 &mut nb_nal as *mut c_int)
        };

        if bytes < 0 {
            Err("Encoding Headers Failed")
        } else {
            let mut data = NalData::with_capacity(bytes as usize);

            for i in 0..nb_nal {
                let nal = unsafe { Box::from_raw(c_nals.offset(i as isize)) };

                let payload =
                    unsafe { std::slice::from_raw_parts(nal.p_payload, nal.i_payload as usize) };

                data.vec.extend_from_slice(payload);

                mem::forget(payload);
                mem::forget(nal);
            }

            Ok(data)
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe { x264_encoder_close(self.enc) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let mut par = Param::new().set_dimension(640, 480);

        let mut enc = Encoder::open(&mut par).unwrap();

        let headers = enc.get_headers().unwrap();

        println!("Headers len {}", headers.as_bytes().len());
    }

    #[test]
    fn test_picture() {
        let par = Param::new().set_dimension(640, 480);
        {
            let mut pic = Picture::from_param(&par).unwrap();
            {
                let p = pic.as_slice_mut(0).unwrap();
                p[0] = 1;
            }
            let p = pic.as_slice(0).unwrap();

            assert_eq!(p[0], 1);
        }
    }
}
