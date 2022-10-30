use crate::include_bytes_align_as;
use crate::{current_time_us, framebuffer::Framebuffer, VideoMode};
use n64_macros::debugln;
use n64_sys::{rdp, rsp, vi};

pub struct Graphics {}

impl Graphics {
    #[inline]
    pub(crate) fn new(video_mode: VideoMode, framebuffer: &mut Framebuffer) -> Self {
        vi::init(video_mode, &mut framebuffer.vi_buffer.0);
        rdp::init();
        rsp::init();
        Self {}
    }

    #[inline]
    pub fn swap_buffers(&mut self, framebuffer: &mut Framebuffer) -> i64 {
        rdp::wait_for_done();

        framebuffer.swap();

        let swap_start = current_time_us();
        vi::wait_for_vblank();
        let swap_end = current_time_us();
        unsafe { vi::set_vi_buffer(&mut framebuffer.vi_buffer.0) };

        swap_end - swap_start
    }

    #[inline]
    pub fn rsp_hello_world(&self) {
        let code = include_bytes_align_as!(u64, "../../n64-sys/rsp/hello_world.bin");

        rsp::run(code, None);

        let mut dmem = [0u8; 4096];

        rsp::read_dmem(&mut dmem);

        for (i, word) in dmem.windows(4).enumerate() {
            let a = u32::from_be_bytes(word.try_into().unwrap());

            debugln!("{} == {}", a, i);

            assert!(a == i as u32);
        }

        panic!("DONE");
    }
}
