use ffmpeg::*;
use libwebp::*;
use std::{io::Write, *};
use stdc::CArray;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
macro_rules! line {
    () => {
        |e| {
            format!(
                "{}:{}:{}",
                panic::Location::caller().file(),
                panic::Location::caller().line(),
                e
            )
        }
    };
}
fn main() {
    let mut fctx: FfmpegCtx = FfmpegCtx::new("test.mp4").unwrap();
    println!("# of Frames: {}", fctx.frame_count().unwrap());
    fctx.init_frame_convert(0, 200, false).unwrap();
    let (_, width, height) = fctx.get_conv_frame_data().map_err(line!()).unwrap();
    let mut wctx = WebpCtx::new(100.0, false, 0, 1, 0, width, height).unwrap();
    let webp_file_data = convert_mp4_webp(&mut fctx, &mut wctx, false).unwrap();
    let mut file = fs::File::create("test.webp").unwrap();
    file.write_all(webp_file_data.get_slice().unwrap()).unwrap();
    file.flush().unwrap();
}

fn convert_mp4_webp(fctx: &mut FfmpegCtx, wctx: &mut WebpCtx, rgb: bool) -> Result<CArray<'static>> {
    let mut timestamp_ms = 0;
    while fctx.read_next_frame() {
        fctx.send_packet(false).map_err(line!())?;
        while fctx.decode_frame().map_err(line!())? {
            fctx.convert_frame().map_err(line!())?;
            let (frame_data,w,h) = fctx.get_conv_frame_data().map_err(line!())?;
            wctx.add_anim_frame(frame_data, w, h, timestamp_ms, rgb, false).map_err(line!())?;
            timestamp_ms += fctx.frame_cleanup();
        }
        fctx.packet_cleanup();
    }
    let (d,w,h) = fctx.get_conv_frame_data().map_err(line!())?;
    wctx.add_anim_frame(d, w, h, timestamp_ms, rgb, true).map_err(line!())?;
    Ok(wctx.get_anim_webp().map_err(line!())?)
}
