use core::{fmt, ptr};

use bootloader_api::{
	info::{FrameBuffer, FrameBufferInfo, PixelFormat},
	BootInfo,
};
use noto_sans_mono_bitmap::{
	get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

use crate::{mutex::Mutex, once_lock::OnceLock};

pub static WRITER: OnceLock<Mutex<FrameBufferWriter>> = OnceLock::new();

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;
/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;
/// Backup character if a desired symbol is not available by the font.
/// The '�' character requires the feature "unicode-specials".
const BACKUP_CHAR: char = '�';

pub fn init_framebuffer(frame: &'static mut FrameBuffer) -> Mutex<FrameBufferWriter> {
	let info = frame.info();

	Mutex::new(FrameBufferWriter::new(frame.buffer_mut(), info))
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char, style: FontWeight, size: RasterHeight) -> RasterizedChar {
	fn get(c: char, style: FontWeight, size: RasterHeight) -> Option<RasterizedChar> {
		get_raster(c, style, size)
	}
	get(c, style, size).unwrap_or_else(|| {
		get(BACKUP_CHAR, style, size).expect("Should get raster of backup char.")
	})
}

/// Allows logging text to a pixel-based framebuffer.
#[derive(Debug)]
pub struct FrameBufferWriter {
	framebuffer: &'static mut [u8],
	lines_history: [usize; 1500],
	info: FrameBufferInfo,
	x_pos: usize,
	y_pos: usize,
	curr_line: usize,
	curr_n_char: usize,
	raster_size: RasterHeight,
	raster_width: usize,
	font_weight: FontWeight,
}

impl FrameBufferWriter {
	/// Creates a new logger that uses the given framebuffer.
	pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
		let raster_size = match info.height {
			0..=800 => RasterHeight::Size16,
			801..=1199 => RasterHeight::Size20,
			1200..=1799 => RasterHeight::Size24,
			1800.. => RasterHeight::Size32,
		};
		let font_weight = FontWeight::Regular;

		let mut logger = Self {
			framebuffer,
			info,
			lines_history: [0; 1500],
			x_pos: 0,
			y_pos: 0,
			curr_line: 0,
			curr_n_char: 0,
			raster_size,
			font_weight,
			raster_width: get_raster_width(font_weight, raster_size),
		};
		logger.clear();
		logger
	}

	fn newline(&mut self) {
		self.y_pos += self.raster_size.val() + LINE_SPACING;
		self.lines_history[self.curr_line] = self.curr_n_char;
		self.curr_line += 1;
		self.curr_n_char = 0;
		self.carriage_return()
	}

	fn carriage_return(&mut self) {
		self.curr_n_char = 0;
		self.x_pos = BORDER_PADDING;
	}

	/// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
	pub fn clear(&mut self) {
		self.curr_n_char = 0;
		self.curr_line = 0;
		self.lines_history.fill(0);
		self.x_pos = BORDER_PADDING;
		self.y_pos = BORDER_PADDING;
		self.framebuffer.fill(0);
	}

	// pub fn make_space(&mut self) {
	//     let dummy_rendered_char =
	//         self.dummy_rendered_char(get_raster(' ', self.font_weight, self.raster_size).unwrap());
	//     let chars = (self.width()) / (self.raster_size.val());

	//     #[cfg(feature = "serial")]
	//     crate::serial_println!("\n{} u {}", chars, dummy_rendered_char);

	//     let offset = 3 * (chars * dummy_rendered_char) - LINE_SPACING * self.width();
	//     self.framebuffer.copy_within(offset.., 0);
	//     let len = self.framebuffer.len();
	//     self.framebuffer[len - offset..].fill(0);
	//     self.curr_n_char = 0;

	//     self.lines_history[self.curr_line..].fill(0);
	//     self.lines_history.copy_within(1..self.curr_line, 0);

	//     self.x_pos = BORDER_PADDING;
	//     // self.y_pos -= LINE_SPACING;
	// }

	fn width(&self) -> usize { self.info.width }

	fn height(&self) -> usize { self.info.height }

	/// Writes a single char to the framebuffer. Takes care of special control characters, such as
	/// newlines and carriage returns.
	fn write_char(&mut self, c: char) {
		match c {
			// '\n' if self.y_pos + self.raster_size.val() + BORDER_PADDING >= self.height() => {
			//     self.newline();
			//     // self.make_space();
			// }
			'\n' => self.newline(),
			'\r' => self.carriage_return(),
			'\u{8}' => {
				let new_xpos = self.x_pos + self.raster_width;
				if new_xpos >= self.width() || new_xpos <= self.raster_width + BORDER_PADDING {
					self.x_pos = if self.curr_line > 0 {
						self.curr_line -= 1;
						self.curr_n_char = self.lines_history[self.curr_line];
						self.raster_width * self.lines_history[self.curr_line]
					} else {
						return;
					};
					self.y_pos -= self.raster_size.val() + LINE_SPACING;
				}
				self.curr_n_char -= 1;
				self.unwrite_rendered_char(get_char_raster(
					' ',
					self.font_weight,
					self.raster_size,
				));
			}
			c => {
				let new_xpos = self.x_pos + self.raster_width;
				let new_ypos = self.y_pos + self.raster_size.val() + BORDER_PADDING;
				if new_ypos >= self.height() {
					self.newline();
					self.clear();
				// self.make_space();
				} else if new_xpos >= self.width() {
					self.newline();
				}
				self.curr_n_char += 1;
				self.write_rendered_char(get_char_raster(c, self.font_weight, self.raster_size));
			}
		}
	}

	/// Prints a rendered char into the framebuffer.
	/// Updates `self.x_pos`.
	fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
		for (y, row) in rendered_char.raster().iter().enumerate() {
			for (x, byte) in row.iter().enumerate() {
				self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
			}
		}
		self.x_pos += rendered_char.width() + LETTER_SPACING;
	}

	fn unwrite_rendered_char(&mut self, rendered_char: RasterizedChar) {
		self.x_pos -= rendered_char.width() + LETTER_SPACING;
		for (y, row) in rendered_char.raster().iter().enumerate() {
			for (x, byte) in row.iter().enumerate() {
				self.write_pixel(self.x_pos + x, self.y_pos + y, 0);
			}
		}
	}

	fn dummy_rendered_char(&mut self, rendered_char: RasterizedChar) -> usize {
		rendered_char.raster().len()
			* rendered_char.raster().first().map(|a| a.len()).unwrap()
			* self.info.bytes_per_pixel
	}

	fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
		let pixel_offset = y * self.info.stride + x;
		let color = match self.info.pixel_format {
			PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
			PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
			PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
			other => {
				// set a supported (but invalid) pixel format before panicking to avoid a double
				// panic; it might not be readable though
				self.info.pixel_format = PixelFormat::Rgb;
				panic!("pixel format {:?} not supported in logger", other)
			}
		};
		let bytes_per_pixel = self.info.bytes_per_pixel;
		let byte_offset = pixel_offset * bytes_per_pixel;
		self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
			.copy_from_slice(&color[..bytes_per_pixel]);
		let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
	}

	pub const fn raster_size(&self) -> usize { self.raster_size.val() }

	pub const fn raster_width(&self) -> usize { self.raster_width }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for c in s.chars() {
			self.write_char(c);
		}
		Ok(())
	}
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::frame::_print(format_args!($($arg)*));
        #[cfg(feature = "serial")]
        $crate::serial_print!($($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	x86_64::instructions::interrupts::without_interrupts(|| {
		// new
		WRITER.get().unwrap().lock().write_fmt(args).unwrap();
	});
}
