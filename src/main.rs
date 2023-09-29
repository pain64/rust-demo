mod field {
    use std::fmt::Debug;

    pub struct Field {
        pub size: i32,
        pub data: Vec<u8>,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Color {
        Black,
        White,
    }

    impl Field {
        pub fn new(size: i32) -> Field {
            let n_cells = size * size;
            let n_bytes = n_cells / 8
                + match n_cells % 8 {
                    0 => 0,
                    _ => 1,
                };

            Field {
                size,
                data: vec![0; n_bytes as usize],
            }
        }

        fn convert_index(&self, i: usize, j: usize) -> (usize, usize) {
            let flat_index = i * (self.size as usize) + j;
            let byte_index = flat_index / 8;
            let bit_index = flat_index % 8;

            (byte_index, bit_index)
        }

        pub fn get(&self, i: i32, j: i32) -> Color {
            let (byte_index, bit_index) = self.convert_index(i as usize, j as usize);

            match self.data[byte_index] & (1 << bit_index) {
                0 => Color::White,
                _ => Color::Black,
            }
        }

        pub fn set(&mut self, i: i32, j: i32, color: Color) {
            let (byte_index, bit_index) = self.convert_index(i as usize, j as usize);
            let old = self.data[byte_index];

            self.data[byte_index] = match color {
                Color::White => old & !(1 << bit_index),
                Color::Black => old | (1 << bit_index),
            };
        }
    }
}

mod bmp {
    use std::{
        fs::File,
        io::{Error, Write},
    };

    pub struct ColorMapEntry {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    pub enum RowOrder {
        TopDown,
        BottomUp,
    }

    fn write_i32(file: &mut File, x: i32) -> Result<(), Error> {
        file.write_all(x.to_le_bytes().as_slice())?;
        Ok(())
    }

    fn write_u32(file: &mut File, x: u32) -> Result<(), Error> {
        file.write_all(x.to_le_bytes().as_slice())?;
        Ok(())
    }

    fn write_u16(file: &mut File, x: u16) -> Result<(), Error> {
        file.write_all(x.to_le_bytes().as_slice())?;
        Ok(())
    }

    fn write_byte(file: &mut File, x: u8) -> Result<(), Error> {
        file.write_all(x.to_le_bytes().as_slice())?;
        Ok(())
    }

    // https://www.fileformat.info/format/bmp/egff.htm

    const FILE_HEADER_SIZE: u32 = 14;
    const DIB_HEADER_SIZE: u32 = 40;

    fn write_bmp_header(file: &mut File, color_map_size: u32) -> Result<(), Error> {
        write_u16(file, 0x4d42)?; /* File type, always 4D42h ("BM") */
        write_u32(file, 0)?; /* Size of the file in bytes, 0 for uncompressed */
        write_u16(file, 0)?; /* Always 0 */
        write_u16(file, 0)?; /* Always 0 */
        /* Starting position of image data in bytes */
        write_u32(
            file,
            FILE_HEADER_SIZE + DIB_HEADER_SIZE + color_map_size * 4,
        )?;

        Ok(())
    }

    fn write_dib3_header(
        file: &mut File,
        width: u32,
        height: u32,
        row_order: RowOrder,
        data_size: u32,
    ) -> Result<(), Error> {
        let fixed_height = match row_order {
            RowOrder::TopDown => height as i32,
            RowOrder::BottomUp => -(height as i32),
        };

        write_u32(file, DIB_HEADER_SIZE)?; /* Size of this header in bytes */
        write_u32(file, width)?; /* Image width in pixels */
        write_i32(file, fixed_height)?; /* Image height in pixels */
        write_u16(file, 1)?; /* Number of color planes */
        write_u16(file, 1)?; /* Number of bits per pixel */
        write_u32(file, 0)?; /* Compression methods used */
        write_u32(file, data_size)?; /* Size of bitmap in bytes */
        write_u32(file, 1000)?; /* Horizontal resolution in pixels per meter */
        write_u32(file, 1000)?; /* Vertical resolution in pixels per meter */
        write_u32(file, 2)?; /* Number of colors in the image */
        write_u32(file, 0)?; /* Minimum number of important colors */

        Ok(())
    }

    fn write_color_map_entry(file: &mut File, entry: &ColorMapEntry) -> Result<(), Error> {
        write_byte(file, entry.b)?;
        write_byte(file, entry.g)?;
        write_byte(file, entry.r)?;
        write_byte(file, 0)?;

        Ok(())
    }

    pub fn write(
        file: &mut File,
        width: u32,
        height: u32,
        color_map: &[ColorMapEntry],
        row_order: RowOrder,
        data: &[u8],
    ) -> Result<(), Error> {
        write_bmp_header(file, color_map.len() as u32)?;
        write_dib3_header(file, width, height, row_order, data.len() as u32)?;

        for entry in color_map {
            write_color_map_entry(file, entry)?;
        }

        // FIXME: row padding
        file.write_all(data)?;

        Ok(())
    }
}

use bmp::{ColorMapEntry, RowOrder};
use field::{Color, Field};
use std::{fs::File, io::Error};

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

fn main() -> Result<(), Error> {
    let mut field = Field::new(1024);
    let mut x: i32 = 512;
    let mut y: i32 = 512;
    let mut direction = Direction::Up;

    fn go(direction: &Direction, x: i32, y: i32) -> (i32, i32) {
        match direction {
            Direction::Left => (x - 1, y),
            Direction::Up => (x, y + 1),
            Direction::Right => (x + 1, y),
            Direction::Down => (x, y - 1),
        }
    }

    while x >= 0 && y >= 0 && x < field.size && y < field.size {
        match field.get(x, y) {
            Color::White => {
                field.set(x, y, Color::Black);
                direction = match direction {
                    Direction::Left => Direction::Up,
                    Direction::Up => Direction::Right,
                    Direction::Right => Direction::Down,
                    Direction::Down => Direction::Left,
                };

                (x, y) = go(&direction, x, y);
            }
            Color::Black => {
                field.set(x, y, Color::White);
                direction = match direction {
                    Direction::Left => Direction::Down,
                    Direction::Up => Direction::Left,
                    Direction::Right => Direction::Up,
                    Direction::Down => Direction::Right,
                };

                (x, y) = go(&direction, x, y);
            }
        }
    }

    let mut n_black = 0;
    for i in 0..field.size {
        for j in 0..field.size {
            if field.get(i, j) == Color::Black {
                n_black += 1;
            }
        }
    }

    println!("number of black cells: {}", n_black);

    let mut file = File::create("ant_path.bmp")?;
    bmp::write(
        &mut file,
        field.size as u32,
        field.size as u32,
        &[
            ColorMapEntry { r: 0, g: 255, b: 0 }, // green background
            ColorMapEntry { r: 0, g: 0, b: 255 }, // blue path
        ],
        RowOrder::BottomUp,
        field.data.as_slice(),
    )?;

    Ok(())
}
