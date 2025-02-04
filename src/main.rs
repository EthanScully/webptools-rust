use ffmpeg;
use std::*;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
fn main() {
    let mut ctx = ffmpeg::new("test.mp4").unwrap();
    println!("# of Frames: {}", ctx.frame_count().unwrap());
    ctx.init_frame_convert(200, 0, false).unwrap();
    let mut timestamp = 0;
    let custom_frame_duration = 0;
    while ctx.read_next_frame() {
        add_frame(&mut ctx, false, &mut timestamp, custom_frame_duration).unwrap();
    }
    add_frame(&mut ctx, true, &mut 0, 0).unwrap();
    // flush webp encoder
    // extract webp file
}
/// Decodes a frames and adds it current WEBP
fn add_frame(ctx: &mut ffmpeg::FfmpegCtx, nil: bool, ts: &mut i32, cfd: i32) -> Result<()> {
    ctx.send_packet(nil)?;
    while ctx.decode_frame()? {
        ctx.convert_frame()?;
        // encode webp frame
        let fd = ctx.frame_cleanup();
        if cfd <= 0 {
            *ts += fd;
        } else {
            *ts += cfd;
        }
    }
    ctx.unref_packet();
    Ok(())
}
