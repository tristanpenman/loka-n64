use core::ptr::{read_volatile, write_volatile};

pub const WIDTH: i32 = 320;
pub const HEIGHT: i32 = 240;
pub const FRAME_BUFFER_SIZE: i32 = WIDTH * HEIGHT * 4;

const FRAME_BUFFER: *mut u16 = (0xA040_0000 - 2 * (FRAME_BUFFER_SIZE as usize) - 4) as *mut u16;

const VI_BASE: usize = 0xA440_0000;

const VI_STATUS: *mut u32 = VI_BASE as *mut u32;
const VI_DRAM_ADDR: *mut usize = (VI_BASE + 0x04) as *mut usize;
const VI_H_WIDTH: *mut u32 = (VI_BASE + 0x08) as *mut u32;
const VI_V_INTR: *mut u32 = (VI_BASE + 0x0C) as *mut u32;
const VI_CURRENT: *const u32 = (VI_BASE + 0x10) as *const u32;
const VI_TIMING: *mut u32 = (VI_BASE + 0x14) as *mut u32;
const VI_V_SYNC: *mut u32 = (VI_BASE + 0x18) as *mut u32;
const VI_H_SYNC: *mut u32 = (VI_BASE + 0x1C) as *mut u32;
const VI_H_SYNC_LEAP: *mut u32 = (VI_BASE + 0x20) as *mut u32;
const VI_H_VIDEO: *mut u32 = (VI_BASE + 0x24) as *mut u32;
const VI_V_VIDEO: *mut u32 = (VI_BASE + 0x28) as *mut u32;
const VI_V_BURST: *mut u32 = (VI_BASE + 0x2C) as *mut u32;
const VI_X_SCALE: *mut u32 = (VI_BASE + 0x30) as *mut u32;
const VI_Y_SCALE: *mut u32 = (VI_BASE + 0x34) as *mut u32;

#[inline]
pub fn init() {
    let frame_buffer = FRAME_BUFFER as usize;
    for i in 0..WIDTH * HEIGHT {
        let p = (frame_buffer + i as usize * 4) as *mut u32;
        unsafe {
            write_volatile(p, 0x0001_0001);
        }
    }

    unsafe {
        write_volatile(VI_STATUS, 0x0000_320E);
        write_volatile(VI_DRAM_ADDR, frame_buffer);
        write_volatile(VI_H_WIDTH, WIDTH as u32);
        write_volatile(VI_V_INTR, 2);
        write_volatile(VI_TIMING, 0x03E5_2239);
        write_volatile(VI_V_SYNC, 0x0000_020D);
        write_volatile(VI_H_SYNC, 0x0000_0C15);
        write_volatile(VI_H_SYNC_LEAP, 0x0C15_0C15);
        write_volatile(VI_H_VIDEO, 0x006C_02EC);
        write_volatile(VI_V_VIDEO, 0x0025_01FF);
        write_volatile(VI_V_BURST, 0x000E_0204);
        write_volatile(VI_X_SCALE, 0x0000_0200);
        write_volatile(VI_Y_SCALE, 0x0000_0400);
    }
}

#[inline]
pub fn wait_for_vblank() {
    loop {
        let current_halfline = unsafe { read_volatile(VI_CURRENT) };
        if current_halfline <= 10 {
            break;
        }
    }
}

#[inline]
pub unsafe fn next_buffer() -> *mut u16 {
    let current_fb = read_volatile(VI_DRAM_ADDR);

    if current_fb != FRAME_BUFFER as usize {
        FRAME_BUFFER
    } else {
        (FRAME_BUFFER as usize + FRAME_BUFFER_SIZE as usize) as *mut u16
    }
}

#[inline]
pub fn swap_buffers() {
    unsafe {
        write_volatile(VI_DRAM_ADDR, next_buffer() as usize);
    }
}