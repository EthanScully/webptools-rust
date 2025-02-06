mod C;
use std::*;
macro_rules! line {
    () => {
        |e| format!("{}:{}:{}", panic::Location::caller().file(), panic::Location::caller().line(), e)
    };
}
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
pub struct FfmpegCtx {
    fmt: *mut C::AVFormatContext,
    codec: *mut C::AVCodecContext,
    pkt: *mut C::AVPacket,
    frame: *mut C::AVFrame,
    /// Extra Frame used for PXL scaling / conversion
    dummy_frame: *mut C::AVFrame,
    /// Prmary Video Stream Index
    index: ffi::c_int,
    /// (width,height)
    resolution: (i32, i32),
    /// Number of Frames in Video
    num_frames: i64,
}
impl FfmpegCtx {
    /// ### initialize ffmpeg environment context
    pub fn new(filepath: &str) -> Result<Self> {
        let mut fmt: *mut C::AVFormatContext = ptr::null_mut();
        let pkt: *mut C::AVPacket;
        let frame: *mut C::AVFrame;
        let dummy_frame: *mut C::AVFrame;
        let codec: *mut C::AVCodecContext;
        let index: ffi::c_int;
        let resolution: (i32, i32);
        let num_frames: i64;
        unsafe {
            if C::avformat_open_input(
                &mut fmt,
                ffi::CString::new(format!("file:{}", filepath)).map_err(line!())?.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            ) < 0
            {
                return Err(format!("error opening input")).map_err(line!())?;
            }
            if C::avformat_find_stream_info(fmt, ptr::null_mut()) < 0 {
                return Err(format!("error finding stream info")).map_err(line!())?;
            }
            index = C::av_find_best_stream(fmt, 0, -1, -1, ptr::null_mut(), 0);
            if index < 0 {
                return Err(format!("could not find stream in input file")).map_err(line!())?;
            }
            let stream = slice::from_raw_parts((*fmt).streams, (index + 1) as usize)[index as usize];
            if stream.is_null() {
                return Err(format!("could not find stream in the input")).map_err(line!())?;
            }
            num_frames = (*stream).nb_frames;
            let dec = C::avcodec_find_decoder((*(*stream).codecpar).codec_id);
            if dec.is_null() {
                return Err(format!("error finding codec")).map_err(line!())?;
            }
            codec = C::avcodec_alloc_context3(dec);
            if codec.is_null() {
                return Err(format!("error allocating codec context")).map_err(line!())?;
            }
            if C::avcodec_parameters_to_context(codec, (*stream).codecpar) < 0 {
                return Err(format!("error copying codec parameters to context")).map_err(line!())?;
            }
            if C::avcodec_open2(codec, dec, ptr::null_mut()) < 0 {
                return Err(format!("error opening codec")).map_err(line!())?;
            }
            resolution = ((*codec).width, (*codec).height);
            frame = C::av_frame_alloc();
            if frame.is_null() {
                return Err(format!("error allocating frame")).map_err(line!())?;
            }
            pkt = C::av_packet_alloc();
            if pkt.is_null() {
                return Err(format!("error allocating packet")).map_err(line!())?;
            }
            dummy_frame = C::av_frame_alloc();
            if frame.is_null() {
                return Err(format!("error allocating frame")).map_err(line!())?;
            }
        }
        Ok(FfmpegCtx {
            fmt,
            codec,
            pkt,
            frame,
            dummy_frame,
            index,
            resolution,
            num_frames,
        })
    }
    pub fn frame_count(&mut self) -> Result<i64> {
        if self.num_frames != 0 {
            return Ok(self.num_frames);
        }
        let mut frames: i64 = 0;
        while self.read_next_frame() {
            frames += 1;
            self.packet_cleanup();
        }
        self.seek_frame(0).map_err(line!())?;
        self.num_frames = frames;
        Ok(frames)
    }
    fn resolution_ratio(&self) -> f64 {
        let (w, h) = self.resolution;
        (w as f64) / (h as f64)
    }
    // Desired resolution and PXL fomat for after decoding
    pub fn init_frame_convert(&mut self, mut width: i32, mut height: i32, rgb: bool) -> Result<()> {
        let pxl_fmt: C::AVPixelFormat;
        if rgb {
            pxl_fmt = C::AVPixelFormat_AV_PIX_FMT_ARGB;
        } else {
            pxl_fmt = C::AVPixelFormat_AV_PIX_FMT_YUV420P;
        }
        if width <= 0 || height <= 0 {
            if width <= 0 && height <= 0 {
                (width, height) = self.resolution;
            } else if width <= 0 {
                width = (height as f64 * self.resolution_ratio()).round() as i32;
            } else {
                height = (width as f64 / self.resolution_ratio()).round() as i32;
            }
        }
        if height % 2 == 1 {
            height -= 1
        }
        if width % 2 == 1 {
            width -= 1
        }
        unsafe {
            (*self.dummy_frame).height = height;
            (*self.dummy_frame).width = width;
            (*self.dummy_frame).format = pxl_fmt;
            let data = (*self.dummy_frame).data.as_mut_ptr();
            if !data.is_null() {
                C::av_freep(data as *mut ffi::c_void);
            }
            let linesizes = (*self.dummy_frame).linesize.as_mut_ptr();
            let ret = C::av_image_alloc(data, linesizes, width, height, pxl_fmt, 1);
            if ret < 0 {
                return Err(format!(
                    "error allocating frame data for conversion frame; av_image_alloc code:{}",
                    ret
                ))
                .map_err(line!())?;
            }
        }
        Ok(())
    }
    pub fn read_next_frame(&mut self) -> bool {
        unsafe {
            loop {
                if C::av_read_frame(self.fmt, self.pkt) >= 0 {
                    if (*self.pkt).stream_index != self.index {
                        C::av_packet_unref(self.pkt);
                        continue;
                    }
                    return true;
                } else {
                    return false;
                }
            }
        }
    }
    pub fn send_packet(&mut self, nil: bool) -> Result<()> {
        let r: i32;
        unsafe {
            if nil {
                r = C::avcodec_send_packet(self.codec, ptr::null())
            } else {
                r = C::avcodec_send_packet(self.codec, self.pkt)
            }
        }
        if r < 0 {
            Err(format!("error sending packet")).map_err(line!())?
        } else {
            Ok(())
        }
    }
    pub fn decode_frame(&mut self) -> Result<bool> {
        unsafe {
            let r = C::avcodec_receive_frame(self.codec, self.frame);
            if r < 0 {
                if r == -541478725 || r == -11 {
                    return Ok(false);
                }
                return Err(format!("error during decoding:{}", r)).map_err(line!())?;
            } else {
                return Ok(true);
            }
        }
    }
    /// Returns the duration of the frame
    pub fn frame_cleanup(&mut self) -> i32 {
        unsafe {
            let stream = slice::from_raw_parts((*self.fmt).streams, (self.index + 1) as usize)[self.index as usize];
            let frame_duration =
                ((*self.frame).duration as f64 * ((*stream).time_base.num as f64 / (*stream).time_base.den as f64) * 1000.0).round() as i32;
            C::av_frame_unref(self.frame);
            return frame_duration;
        }
    }
    pub fn packet_cleanup(&mut self) {
        unsafe {
            C::av_packet_unref(self.pkt);
        }
    }
    pub fn convert_frame(&mut self) -> Result<()> {
        unsafe {
            let sws_ctx = C::sws_getContext(
                (*self.frame).width,
                (*self.frame).height,
                (*self.frame).format,
                (*self.dummy_frame).width,
                (*self.dummy_frame).height,
                (*self.dummy_frame).format,
                C::SWS_BICUBIC as i32,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            );
            if sws_ctx.is_null() {
                return Err(format!("error getting SwsContext")).map_err(line!())?;
            }
            let r = C::sws_scale(
                sws_ctx,
                (*self.frame).data.as_ptr() as *const *const u8,
                (*self.frame).linesize.as_ptr(),
                0,
                (*self.frame).height,
                (*self.dummy_frame).data.as_ptr(),
                (*self.dummy_frame).linesize.as_ptr(),
            );
            if r < 0 {
                return Err(format!("error scaling frame")).map_err(line!())?;
            }
            C::sws_freeContext(sws_ctx);
            return Ok(());
        }
    }
    /// UNSAFE return value is only valid until convert_frame() is ran again
    /// returns (&data, width, height)
    pub fn get_conv_frame_data(&self) -> Result<(&[*mut u8; 8], i32, i32)> {
        unsafe {
            let (w, h) = ((*self.dummy_frame).width, (*self.dummy_frame).height);
            if w == 0 || h == 0 {
                Err(format!("data doesn't exist")).map_err(line!())?
            }
            Ok((&(*self.dummy_frame).data, w, h))
        }
    }
    /// Get RGB data from single frame
    pub fn retrieve_single_frame(&mut self, frame_num: i32, width: i32, height: i32) -> Result<&[u8]> {
        let output: &[u8];
        self.init_frame_convert(width, height, true).map_err(line!())?;
        self.seek_frame(frame_num as i64).map_err(line!())?;
        while self.read_next_frame() {
            self.send_packet(false).map_err(line!())?;
            while self.decode_frame().map_err(line!())? {
                self.convert_frame().map_err(line!())?;
                unsafe {
                    let len = (*self.dummy_frame).linesize[0] as usize * (*self.dummy_frame).height as usize;
                    output = slice::from_raw_parts((*self.dummy_frame).data[0], len);
                }
                let _ = self.frame_cleanup();
                self.packet_cleanup();
                self.seek_frame(0).map_err(line!())?;
                return Ok(output);
            }
            self.packet_cleanup();
        }
        return Err(format!("error decoding given frame")).map_err(line!())?;
    }
    pub fn seek_frame(&mut self, frame_num: i64) -> Result<()> {
        if frame_num as i64 >= self.frame_count().map_err(line!())? {
            return Err(format!("selected frame is larger than amount in given media")).map_err(line!())?;
        }
        unsafe {
            if C::av_seek_frame(self.fmt, self.index, frame_num, C::AVSEEK_FLAG_FRAME as i32) >= 0 {
                return Ok(());
            }
            let stream = slice::from_raw_parts((*self.fmt).streams, self.index as usize + 1)[self.index as usize];
            let time_base = (*stream).time_base;
            let framerate = C::av_guess_frame_rate(self.fmt, stream, self.frame);
            let inv_framerate = C::AVRational {
                num: framerate.den,
                den: framerate.num,
            };
            let timestamp = C::av_rescale_q(frame_num, inv_framerate, time_base);
            if C::av_seek_frame(self.fmt, self.index, timestamp, C::AVSEEK_FLAG_BACKWARD as i32) < 0 {
                Err(format!("error seeking to frame: {}", frame_num)).map_err(line!())?;
            }
            C::avcodec_flush_buffers(self.codec);
            Ok(())
        }
    }
}
impl Drop for FfmpegCtx {
    fn drop(&mut self) {
        unsafe {
            let data = (*self.dummy_frame).data.as_mut_ptr();
            if !data.is_null() {
                C::av_freep(data as *mut ffi::c_void);
            }
            self.send_packet(true).map_err(line!()).unwrap();
            C::av_frame_free(&mut self.dummy_frame);
            C::av_frame_free(&mut self.frame);
            C::av_packet_free(&mut self.pkt);
            C::avcodec_free_context(&mut self.codec);
            C::avformat_close_input(&mut self.fmt);
        }
    }
}
