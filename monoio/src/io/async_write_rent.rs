use crate::{
    buf::{IoBuf, IoBufMut, IoVecBuf},
    BufResult,
};
use std::future::Future;

/// AsyncWriteRent: async write with a ownership of a buffer
pub trait AsyncWriteRent {
    /// The future of write Result<size, buffer>
    type WriteFuture<'a, T>: Future<Output = BufResult<usize, T>>
    where
        Self: 'a,
        T: 'a;
    /// The future of writev Result<size, buffer>
    type WritevFuture<'a, T>: Future<Output = BufResult<usize, T>>
    where
        Self: 'a,
        T: 'a;

    /// The future of flush
    type FlushFuture<'a>: Future<Output = std::io::Result<()>>
    where
        Self: 'a;

    /// The future of shutdown
    type ShutdownFuture<'a>: Future<Output = std::io::Result<()>>
    where
        Self: 'a;

    /// Same as write(2)
    fn write<T: IoBuf>(&mut self, buf: T) -> Self::WriteFuture<'_, T>;

    /// Same as writev(2)
    fn writev<T: IoVecBuf>(&mut self, buf_vec: T) -> Self::WritevFuture<'_, T>;

    /// Flush buffered data if needed
    fn flush(&mut self) -> Self::FlushFuture<'_>;

    /// Same as shutdown
    fn shutdown(&mut self) -> Self::ShutdownFuture<'_>;
}

/// AsyncWriteRentAt: async write with a ownership of a buffer and a position
pub trait AsyncWriteRentAt {
    /// The future of Result<size, buffer>
    type Future<'a, T>: Future<Output = BufResult<usize, T>>
    where
        Self: 'a,
        T: 'a;

    /// Write buf at given offset
    fn write_at<T: IoBufMut>(&self, buf: T, pos: usize) -> Self::Future<'_, T>;
}

impl<A: ?Sized + AsyncWriteRent> AsyncWriteRent for &mut A {
    type WriteFuture<'a, T> = A::WriteFuture<'a, T>
    where
        Self: 'a,
        T: 'a;

    type WritevFuture<'a, T> = A::WritevFuture<'a, T>
    where
        Self: 'a,
        T: 'a;

    type FlushFuture<'a> = A::FlushFuture<'a>
    where
        Self: 'a;

    type ShutdownFuture<'a> = A::ShutdownFuture<'a>
    where
        Self: 'a;

    fn write<T: IoBuf>(&mut self, buf: T) -> Self::WriteFuture<'_, T> {
        (&mut **self).write(buf)
    }

    fn writev<T: IoVecBuf>(&mut self, buf_vec: T) -> Self::WritevFuture<'_, T> {
        (&mut **self).writev(buf_vec)
    }

    fn flush(&mut self) -> Self::FlushFuture<'_> {
        (&mut **self).flush()
    }

    fn shutdown(&mut self) -> Self::ShutdownFuture<'_> {
        (&mut **self).shutdown()
    }
}
