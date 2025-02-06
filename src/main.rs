use ffmpeg::*;
//use libwebp::*;
use std::*;
//type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
fn main() {
    let mut ctx: FfmpegCtx = FfmpegCtx::new("test.mp4").unwrap();
    println!("# of Frames: {}", ctx.frame_count().unwrap());
    let _frame = ctx.retrieve_single_frame(0, 0, 200).unwrap();
    drop(_frame);
    let (w, h) = ctx.get_width_height_dummy_frame();
    println!("w:{w},h:{h}",);
}
