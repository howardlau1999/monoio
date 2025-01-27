use crate::{
    buf::RawBuf,
    io::{AsyncReadRent, AsyncWriteRent},
};

/// Wrapped IO with given read prefix.
pub struct PrefixedReadIo<I, P> {
    io: I,
    prefix: P,

    prefix_finished: bool,
}

impl<I, P> PrefixedReadIo<I, P> {
    /// Create a PrefixedIo with given io and read prefix.
    pub fn new(io: I, prefix: P) -> Self {
        Self {
            io,
            prefix,
            prefix_finished: false,
        }
    }

    /// If the prefix has read to eof
    pub fn prefix_finished(&self) -> bool {
        self.prefix_finished
    }

    /// Into inner
    pub fn into_inner(self) -> I {
        self.io
    }
}

impl<I: AsyncReadRent, P: std::io::Read> AsyncReadRent for PrefixedReadIo<I, P> {
    type ReadFuture<'a, T> = impl std::future::Future<Output = crate::BufResult<usize, T>>
    where
        T: 'a, Self: 'a;

    type ReadvFuture<'a, T> = impl std::future::Future<Output = crate::BufResult<usize, T>>
    where
        T: 'a, Self: 'a;

    fn read<T: crate::buf::IoBufMut>(&mut self, mut buf: T) -> Self::ReadFuture<'_, T> {
        async move {
            if buf.bytes_total() == 0 {
                return (Ok(0), buf);
            }
            if !self.prefix_finished {
                let slice = unsafe {
                    &mut *std::ptr::slice_from_raw_parts_mut(buf.write_ptr(), buf.bytes_total())
                };
                match self.prefix.read(slice) {
                    Ok(0) => {
                        // prefix finished
                        self.prefix_finished = true;
                    }
                    Ok(n) => {
                        unsafe { buf.set_init(n) };
                        return (Ok(n), buf);
                    }
                    Err(e) => {
                        return (Err(e), buf);
                    }
                }
            }
            // prefix eof now, read io directly
            self.io.read(buf).await
        }
    }

    fn readv<T: crate::buf::IoVecBufMut>(&mut self, mut buf: T) -> Self::ReadvFuture<'_, T> {
        async move {
            let n = match unsafe { RawBuf::new_from_iovec_mut(&mut buf) } {
                Some(raw_buf) => self.read(raw_buf).await.0,
                None => Ok(0),
            };
            if let Ok(n) = n {
                unsafe { buf.set_init(n) };
            }
            (n, buf)
        }
    }
}

impl<I: AsyncWriteRent, P> AsyncWriteRent for PrefixedReadIo<I, P> {
    type WriteFuture<'a, T> = I::WriteFuture<'a, T> where
    T: 'a, Self: 'a;

    type WritevFuture<'a, T>= I::WritevFuture<'a, T> where
    T: 'a, Self: 'a;

    type FlushFuture<'a> = I::FlushFuture<'a> where Self: 'a;

    type ShutdownFuture<'a> = I::ShutdownFuture<'a> where Self: 'a;

    fn write<T: crate::buf::IoBuf>(&mut self, buf: T) -> Self::WriteFuture<'_, T> {
        self.io.write(buf)
    }

    fn writev<T: crate::buf::IoVecBuf>(&mut self, buf_vec: T) -> Self::WritevFuture<'_, T> {
        self.io.writev(buf_vec)
    }

    fn flush(&mut self) -> Self::FlushFuture<'_> {
        self.io.flush()
    }

    fn shutdown(&mut self) -> Self::ShutdownFuture<'_> {
        self.io.shutdown()
    }
}
