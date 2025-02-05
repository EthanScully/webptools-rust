use ffmpeg;
use std::*;
//type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
fn main() {
    let mut ctx: ffmpeg::FfmpegCtx = ffmpeg::new("test.mp4").unwrap();
    println!("# of Frames: {}", ctx.frame_count().unwrap());
    let (_,w,h) = ctx.retrieve_single_frame(15, 0, 200).unwrap();
    println!("w:{w},h:{h}", );

}
