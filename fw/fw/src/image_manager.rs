use embedded_graphics::prelude::Size;
use crate::bin_image::BinImage;

pub struct ImageManager {
    images: &'static [u8]
}

impl ImageManager {
    pub fn new(images: &'static [u8]) -> Self {
        ImageManager{images}
    }
    pub fn layout(&self) -> BinImage {
        // Layout image is 480x228, bw, at position 0
        let size = Size::new(480, 228);

        self.get_bw_image(size, 0)
    }

    pub fn big_digit(&self, value: u8) -> BinImage {
        // Big digit image is 80x48, bw, 10 entries starting from position 1, first entry is for 0
        let size = Size::new(80, 148);

        self.get_bw_image(size, (value + 1) as usize)
    }

    pub fn small_digit(&self, value: u8) -> BinImage {
        // Small digit image is 16x16, bw, 10 entries starting from position 11, first entry is for 0
        let size = Size::new(16, 16);

        self.get_bw_image(size, (value + 11) as usize)
    }

    pub fn month(&self, value: u8) -> BinImage {
        // Month image is 168x28, bw, 12 entries starting from position 21, first entry is for January
        let size = Size::new(168, 28);

        self.get_bw_image(size, (value + 21) as usize)
    }

    pub fn weekday(&self, value: u8) -> BinImage {
        // Weekday image is 192x30, bw, 7 entries starting from position 33, first entry is for Monday
        let size = Size::new(192, 30);

        self.get_bw_image(size, (value + 33) as usize)
    }

    pub fn moon(&self, value: u8) -> BinImage {
        // Weekday image is 16x16, bw, 8 entries starting from position 40, first entry is for January
        let size = Size::new(16, 16);

        self.get_bw_image(size, (value + 40) as usize)
    }

    pub fn b_side(&self, value: u16) -> BinImage {
        // B side image is 480x648, bw, 366 entries starting from position 40, first entry is for January
        let size = Size::new(480, 648);
        let day_offset = value * 3;
        self.get_bw_image(size, (48 + day_offset + 2) as usize) //Where 2 is b_side offset in the day triplet
    }

    pub fn a_side(&self, value: u16) -> BinImage {
        // A side image is 480x420, rbw, 366 entries starting from position 40, first entry is for January
        let size = Size::new(480, 420);
        let day_offset = value * 3;
        let bw_data = self.fetch_image_data(48 + day_offset as usize).expect("BW image missing");
        let rw_data = self.fetch_image_data((48 + day_offset+1) as usize);
        BinImage::from_slice(size, bw_data, rw_data)
    }
    fn fetch_image_data(&self, index: usize) -> Option<&'static [u8]> {
        let index_position = index * 4;
        let mut offset_bytes : [u8; 4] = [0,0,0,0];
        offset_bytes.copy_from_slice(&self.images[index_position..index_position+4]);
        let offset = u32::from_le_bytes(offset_bytes) as usize;
        if offset == 0 {
            return None
        }
        let mut image_size_bytes : [u8; 2] = [0,0];
        image_size_bytes.copy_from_slice(&self.images[offset..offset+2]);
        let image_size = u16::from_le_bytes(image_size_bytes) as usize;
        Some(&self.images[offset+2..offset+2+image_size])
    }

    fn get_bw_image(&self, size: Size, index: usize) -> BinImage {
        let image_data = self.fetch_image_data(index).expect("BW image missing");
        BinImage::from_slice(size, image_data, None)
    }
}