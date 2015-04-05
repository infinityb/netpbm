use super::PpmLoadResult;


pub struct PpmPixelChunks<I> where I: Iterator<Item=PpmLoadResult<u32>> {
    iterator: I,
    cur: usize,
    state: [u32; 3],
}


impl<I> Iterator for PpmPixelChunks<I> where I: Iterator<Item=PpmLoadResult<u32>> {
    type Item = PpmLoadResult<[u32; 3]>;

    fn next(&mut self) -> Option<PpmLoadResult<[u32; 3]>> {
        while self.cur < self.state.len() {
            match self.iterator.next() {
                Some(Ok(t)) => {
                    self.state[self.cur] = t;
                    self.cur += 1;
                },
                Some(Err(err)) => return Some(Err(err)),
                None => return None,
            }
        }
        let retval = Some(Ok(self.state));
        self.cur = 0;
        self.state = [0, 0, 0];
        retval
    }
}


pub fn chunks<I: Iterator<Item=PpmLoadResult<u32>>>(iterator: I) -> PpmPixelChunks<I> {
    PpmPixelChunks {
        iterator: iterator,
        cur: 0,
        state: [0, 0, 0],
    }
}

