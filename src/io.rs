use std::io::{
    sink,
    copy,
    ErrorKind,
    Result as IOResult,
    prelude::*,
};

pub(crate) struct CountingReader<R> {
    inner: R,
    count: u64,
}

impl <R: Read> CountingReader<R> {
    pub fn skip_to(&mut self, offset: u64) -> IOResult<u64> {
        let Self { count, .. } = self;
        if *count > offset {
            Err(ErrorKind::Unsupported)?;
        }
        let diff = (offset - *count) as u64;
        if diff < 1 {
            return Ok(0);
        }
        let mut take = self.take(diff);
        copy(&mut take, &mut sink())
    }
}

impl <R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        let Self { inner, count } = self;
        let bytes = inner.read(buf)?;
        *count += bytes as u64;
        Ok(bytes)
    }
}

pub(crate) trait ReadExt<R: Read> {
    fn counting(self) -> CountingReader<R>;
}

impl <R: Read> ReadExt<R> for R {
    fn counting(self) -> CountingReader<R> {
        CountingReader {
            inner: self,
            count: 0,
        }
    }
}
