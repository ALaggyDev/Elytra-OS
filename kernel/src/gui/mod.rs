use core::cell::UnsafeCell;

use alloc::rc::Rc;
use bootloader_api::{BootInfo, info::FrameBuffer};
use spin::Mutex;

use crate::{
    color,
    gui::{
        image::QoiImage,
        structure::{AABB, Color, FrameBufferWriter},
    },
    printlnk,
    user::{sched, task::Task},
};

pub mod image;
pub mod structure;

// I am too lazy to make Task::create_kernel_task able to pass arguments right now.
// So we just use a temporary static variable as a way to pass arguments for now.
static TEMP_FRAMEBUFFER: Mutex<Option<FrameBuffer>> = Mutex::new(None);

pub fn init(boot_info: &mut BootInfo) {
    if let Some(framebuffer) = boot_info.framebuffer.take() {
        printlnk!("Framebuffer found! Initializing GUI...");

        TEMP_FRAMEBUFFER.lock().replace(framebuffer);

        let gui_thread = Rc::new(UnsafeCell::new(Task::create_kernel_task(gui_thread_entry)));
        // GUI thread will start after the sched::begin_scheduler() is called
        unsafe { sched::add_new_task(gui_thread) }
    }
}

fn gui_thread_entry() -> ! {
    printlnk!("GUI thread started.");

    let framebuffer = TEMP_FRAMEBUFFER
        .lock()
        .take()
        .expect("Framebuffer should be set.");

    let mut framebuffer_writer = FrameBufferWriter::new(framebuffer);
    let mut window = framebuffer_writer.create_new_window();

    // Read wallpaper QOI image
    let wallpaper = QoiImage::new(include_bytes!("../../wallpaper.qoi")).unwrap();
    printlnk!(
        "Decoded wallpaper QOI image: width={}, height={}, channels={:?}",
        wallpaper.header().width,
        wallpaper.header().height,
        wallpaper.header().channels
    );

    let window_aabb = window.aabb();

    let color_1 = color!(#03cacd);
    let color_2 = color!(#18f523);
    let mut toggle = false;
    let mut num_frames = 0;

    loop {
        // Draw wallpaper
        wallpaper.draw_to_window(&mut window, window_aabb);

        // Draw taskbar
        let taskbar_height = 40;
        let taskbar_aabb = AABB {
            pos: (0, window_aabb.size.1 - taskbar_height),
            size: (window_aabb.size.0, taskbar_height),
        };
        window.set_pixels(taskbar_aabb, Color::WHITE);

        // Draw something
        let new_aabb = AABB {
            pos: (window_aabb.pos.0 + 100, window_aabb.pos.1 + 100),
            size: (50, 50),
        };
        window.set_pixels(new_aabb, if toggle { color_1 } else { color_2 });
        toggle = !toggle;

        // Copy to framebuffer
        framebuffer_writer.write_to_framebuffer(&window);

        num_frames += 1;
        if num_frames % 50 == 0 {
            printlnk!("GUI frames rendered: {}", num_frames);
        }

        // Yield to other tasks
        unsafe { sched::yield_task() }
    }
}
