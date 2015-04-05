use std::io::{self, Read};
use std::iter::Peekable;
use std::iter::Iterator;
use super::{PpmLoadResult, PpmLoadError};


#[inline(always)]
fn is_whitespace(byte: u8) -> bool {
    byte == b' ' || byte == b'\n'
}


#[inline(always)]
fn is_number(byte: u8) -> bool {
    b'0' <= byte && byte <= b'9'
}


pub fn consume_whitespace<I>(peekable: &mut Peekable<I>) -> PpmLoadResult<()>
    where
        I: Iterator<Item=io::Result<u8>> {

    loop {
        match peekable.peek() {
            Some(&Ok(byte)) if is_whitespace(byte) => peekable.next().unwrap().unwrap(),
            Some(&Ok(byte)) if is_number(byte) => break,
            Some(&Ok(_)) => return Err(PpmLoadError::FormatError),
            Some(&Err(_)) => return Err(PpmLoadError::Io(peekable.next().unwrap().err().unwrap())),
            None => return Ok(()),
        };
    }
    Ok(())
}

pub fn read_number<I>(peekable: &mut Peekable<I>, buf: &mut String) -> PpmLoadResult<()>
    where
        I: Iterator<Item=io::Result<u8>> {

    use std::char::from_u32;
    loop {
        match peekable.peek() {
            Some(&Ok(byte)) if is_whitespace(byte) => break,
            Some(&Ok(byte)) if is_number(byte) => {
                let byte = peekable.next().unwrap().unwrap();
                buf.push(from_u32(byte as u32).unwrap());
            },
            Some(&Ok(_)) => return Err(PpmLoadError::FormatError),
            Some(&Err(_)) => return Err(PpmLoadError::Io(peekable.next().unwrap().err().unwrap())),
            None => return Ok(()),
        };
    }
    Ok(())
}


pub struct PpmChannelValues<R> where R: Read {
    bytes: Peekable<io::Bytes<R>>,
    is_finished: bool,
}


impl<R> PpmChannelValues<R> where R: Read {
    pub fn new(bytes: Peekable<io::Bytes<R>>) -> PpmChannelValues<R> {
        PpmChannelValues {
            bytes: bytes,
            is_finished: false,
        }
    }
}


impl<R> Iterator for PpmChannelValues<R> where R: Read {
    type Item = PpmLoadResult<u32>;

    fn next(&mut self) -> Option<PpmLoadResult<u32>> {
        if self.is_finished {
            return None;
        }

        if let Err(err) = consume_whitespace(&mut self.bytes) {
            return Some(Err(err));
        }

        let mut output: u32 = 0;
        let mut emit_number = false;
        loop {
            match self.bytes.next() {
                Some(Ok(digit)) if is_number(digit) => {
                    emit_number |= true;
                    output *= 10;
                    output += (digit - b'0') as u32;
                },
                Some(Ok(digit)) if is_whitespace(digit) => return Some(Ok(output)),
                Some(Ok(_)) => {
                    self.is_finished = true;
                    return Some(Err(PpmLoadError::FormatError));
                }
                Some(Err(err)) => {
                    self.is_finished = true;
                    return Some(Err(PpmLoadError::Io(err)));
                }
                None if emit_number => return Some(Ok(output)),
                None => return None
            }
        }
    }
}




#[cfg(test)]
mod tests {
    use std::io::{self, Read};
    use super::PpmChannelValues;

    pub fn ppm_channel_values<R: Read>(reader: R) -> PpmChannelValues<R> {
        PpmChannelValues::new(reader.bytes().peekable())
    }

    #[test]
    fn test_p3() {
        let msg = b"\n 12 \n12  4444 44 4444 11 2 3 13  \n  44 \n\n4\n1\n";
        let reader = io::Cursor::new(&msg[..]);
        let values: Vec<_> = ppm_channel_values(reader).map(|v| v.unwrap()).collect();

        assert_eq!(values[0], 12);
        assert_eq!(values[1], 12);
        assert_eq!(values[2], 4444);
        assert_eq!(values[3], 44);
        assert_eq!(values[4], 4444);
        assert_eq!(values[5], 11);
        assert_eq!(values[6], 2);
        assert_eq!(values[7], 3);
        assert_eq!(values[8], 13);
        assert_eq!(values[9], 44);
        assert_eq!(values[10], 4);
        assert_eq!(values[11], 1);
    }

}
