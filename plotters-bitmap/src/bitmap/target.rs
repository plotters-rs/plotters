#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
use std::path::Path;
use std::marker::PhantomData;
#[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
use crate::gif_support;

pub(super) enum Target<'a> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    File(&'a Path),
    Buffer(PhantomData<&'a u32>),
    #[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
    Gif(Box<gif_support::GifFile>),
}

pub(super) enum Buffer<'a> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    Owned(Vec<u8>),
    Borrowed(&'a mut [u8]),
}

impl<'a> Buffer<'a> {
    #[inline(always)]
    pub(super) fn borrow_buffer(&mut self) -> &mut [u8] {
        match self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
            Buffer::Owned(buf) => &mut buf[..],
            Buffer::Borrowed(buf) => *buf,
        }
    }
}

