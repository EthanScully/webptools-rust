mod C;
use std::*;
use stdc::CArray;
macro_rules! line {
    () => {
        |e| format!("{}:{}:{}", panic::Location::caller().file(), panic::Location::caller().line(), e)
    };
}
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
pub struct WebpCtx {
    enc: *mut C::WebPAnimEncoder,
    config: C::WebPConfig,
}
impl WebpCtx {
    pub fn new(quality: f32, lossless: bool, speed: i32, passes: i32, target_size: i32, width: i32, height: i32) -> Result<Self> {
        let mut config: C::WebPConfig;
        let enc: *mut C::WebPAnimEncoder;
        unsafe {
            config = mem::zeroed();
            if C::WebPConfigInitInternal(
                &mut config,
                C::WebPPreset_WEBP_PRESET_DEFAULT,
                75.0,
                C::WEBP_ENCODER_ABI_VERSION as i32,
            ) != 1
            {
                return Err(format!("WebPConfigInit failed")).map_err(line!())?;
            }
            config.quality = quality;
            if lossless {
                config.lossless = 1;
            } else {
                config.lossless = 0;
            }
            config.method = speed;
            config.pass = passes;
            if target_size > 0 {
                config.target_size = target_size;
                config.quality = 100.0;
            }
            enc = C::WebPAnimEncoderNewInternal(width, height, ptr::null(), C::WEBP_MUX_ABI_VERSION as i32);
            if enc.is_null() {
                return Err(format!("error initializing encoder: memory error")).map_err(line!())?;
            }
        }
        Ok(WebpCtx { enc, config })
    }
    pub fn add_anim_frame(
        &mut self,
        frame_data: Option<&[*mut u8; 8]>,
        width: i32,
        height: i32,
        timestamp_ms: i32,
        rgb: bool,
    ) -> Result<()> {
        let frame_data = match frame_data {
            None => {
                unsafe {
                    if C::WebPAnimEncoderAdd(self.enc, ptr::null_mut(), timestamp_ms, &self.config) != 1 {
                        return Err(format!("error flushing encoder")).map_err(line!())?;
                    }
                }
                return Ok(());
            }
            Some(s) => s,
        };
        let mut pic: C::WebPPicture;
        unsafe {
            pic = mem::zeroed();
            if C::WebPPictureInitInternal(&mut pic, C::WEBP_ENCODER_ABI_VERSION as i32) != 1 {
                return Err(format!("WebPPictureInit failed")).map_err(line!())?;
            }
            pic.width = width;
            pic.height = height;
            if rgb {
                pic.use_argb = 1;
            } else {
                pic.use_argb = 0;
            }
            if C::WebPPictureAlloc(&mut pic) != 1 {
                return Err(format!("WebPPictureAlloc failed")).map_err(line!())?;
            }
            if rgb {
                pic.argb = frame_data[0] as *mut u32;
                pic.argb_stride = width;
            } else {
                pic.colorspace = C::WebPEncCSP_WEBP_YUV420;
                pic.y = frame_data[0];
                pic.u = frame_data[1];
                pic.v = frame_data[2];
                pic.y_stride = width;
                pic.uv_stride = width / 2;
            }
            if C::WebPAnimEncoderAdd(self.enc, &mut pic, timestamp_ms, &self.config) != 1 {
                return Err(format!("error adding frame")).map_err(line!())?;
            }
            C::WebPPictureFree(&mut pic)
        }
        Ok(())
    }
    /// returns animated webp file in a C array
    pub fn get_anim_webp(&mut self) -> Result<CArray> {
        let mut output: C::WebPData;
        let c_array: CArray;
        unsafe {
            output = mem::zeroed();
            if C::WebPAnimEncoderAssemble(self.enc, &mut output) != 1 {
                C::WebPFree(output.bytes as *mut ffi::c_void);
                return Err(format!("error assembling output")).map_err(line!())?;
            }
            c_array = CArray::new(output.bytes as *mut u8, output.size)
        }
        Ok(c_array)
    }
}
impl Drop for WebpCtx {
    fn drop(&mut self) {
        unsafe {
            C::WebPAnimEncoderDelete(self.enc);
        }
    }
}
