use std::convert::AsRef;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

mod parse;
mod helpers;


#[derive(Debug)]
pub enum PpmLoadError {
    FormatError,
    OverflowError,
    Truncated,
    Unknown,
    Io(io::Error)
}


impl From<io::Error> for PpmLoadError {
    fn from(err: io::Error) -> PpmLoadError {
        PpmLoadError::Io(err)
    }
}


pub type PpmLoadResult<T> = Result<T, PpmLoadError>;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct PpmPixel(pub u32, pub u32, pub u32);


pub trait FromPpm {
    fn from_ppm(width: u32, height: u32, depth: u32,
                pixels: &mut Iterator<Item=PpmLoadResult<PpmPixel>>
               ) -> PpmLoadResult<Self>;
}


pub fn read_ppm<R, T>(mut reader: R) -> Result<T, PpmLoadError>
    where
        R: Read,
        T: FromPpm {
    
    // TODO(sell): Is this OK?
    let mut header: [u8; 2] = [0, 0];
    let header_read = try!(reader.read(&mut header));
    if header_read != 2 {
        return Err(PpmLoadError::Truncated);
    }

    if &header[..] != b"P3" {
        return Err(PpmLoadError::FormatError);
    }

    let mut values = parse::PpmChannelValues::new(reader.bytes().peekable());
    let width: u32 = try!(values.next().unwrap_or(Err(PpmLoadError::Truncated)));
    let height: u32 = try!(values.next().unwrap_or(Err(PpmLoadError::Truncated)));
    let depth: u32 = try!(values.next().unwrap_or(Err(PpmLoadError::Truncated)));

    let mut pixels = helpers::chunks(values)
        .map(|triple_res| triple_res.map(|triple| PpmPixel(triple[0], triple[1], triple[2])));
        
    FromPpm::from_ppm(width, height, depth, &mut pixels)
}


pub fn load_ppm<T, P>(path: P) -> Result<T, PpmLoadError>
    where
        T: FromPpm,
        P: AsRef<Path> {

    read_ppm(try!(File::open(path)))
}



#[cfg(test)]
mod tests {
    use super::{read_ppm, PpmPixel, PpmLoadResult, FromPpm};
    use std::io;

    struct MockImageType {
        width: u32,
        height: u32,
        pixels: Vec<PpmPixel>,
    }

    impl FromPpm for MockImageType {
        fn from_ppm(width: u32, height: u32, _depth: u32,
                    pixels: &mut Iterator<Item=PpmLoadResult<PpmPixel>>
                   ) -> PpmLoadResult<MockImageType> {

            let mut pixel_buf = Vec::with_capacity((width * height) as usize);
            for pixel in pixels {
                pixel_buf.push(try!(pixel));
            }

            Ok(MockImageType {
                width: width,
                height: height,
                pixels: pixel_buf,
            })
        }
    }

    #[test]
    fn test_p3_mock_image() {
        let msg = b"P3\n3 4 255\n
             77 240 254  44 195  39  57  85 152  80 159 188
            164 165 253 161 114 242  69  63  89  33 160 214
            196 139   2 159 164  51 144  70  69  90  55 133";

        let image: MockImageType = read_ppm(io::Cursor::new(&msg[..])).unwrap();
        assert_eq!(image.width, 3);
        assert_eq!(image.height, 4);
        assert_eq!(image.pixels[0],  PpmPixel( 77, 240, 254));
        assert_eq!(image.pixels[1],  PpmPixel( 44, 195,  39));
        assert_eq!(image.pixels[2],  PpmPixel( 57,  85, 152));
        assert_eq!(image.pixels[3],  PpmPixel( 80, 159, 188));
        assert_eq!(image.pixels[4],  PpmPixel(164, 165, 253));
        assert_eq!(image.pixels[5],  PpmPixel(161, 114, 242));
        assert_eq!(image.pixels[6],  PpmPixel( 69,  63,  89));
        assert_eq!(image.pixels[7],  PpmPixel( 33, 160, 214));
        assert_eq!(image.pixels[8],  PpmPixel(196, 139,   2));
        assert_eq!(image.pixels[9],  PpmPixel(159, 164,  51));
        assert_eq!(image.pixels[10], PpmPixel(144,  70,  69));
        assert_eq!(image.pixels[11], PpmPixel( 90,  55, 133));
    }
}