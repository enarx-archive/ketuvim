use std::borrow::{Borrow, BorrowMut};
use std::io::{Error, Result};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::os::raw::{c_int, c_void};
use std::os::unix::io::{AsRawFd, RawFd};
use std::slice::{from_raw_parts, from_raw_parts_mut, SliceIndex};

use bitflags::bitflags;

pub enum Access {
    Shared,
    Private,
}

bitflags! {
    #[derive(Default)]
    pub struct Protection: c_int {
        const READ = libc::PROT_READ;
        const WRITE = libc::PROT_WRITE;
        const EXECUTE = libc::PROT_EXEC;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Flags: c_int {
        const ANONYMOUS = libc::MAP_ANONYMOUS;
    }
}

pub struct Builder<T: 'static + Copy> {
    phantom: PhantomData<T>,
    addr: libc::uintptr_t,
    extra: usize,
    prot: Protection,
    access: Access,
    flags: Flags,
    fd: RawFd,
    offset: libc::off_t,
}

pub struct Map<T: 'static + Copy>(*mut T, usize);

impl<T: 'static + Copy> Drop for Map<T> {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.0 as *mut c_void, self.1);
        }
    }
}

impl<T: 'static + Copy> Deref for Map<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T: 'static + Copy> DerefMut for Map<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T: 'static + Copy> AsRef<T> for Map<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T: 'static + Copy> AsMut<T> for Map<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T: 'static + Copy> Borrow<T> for Map<T> {
    fn borrow(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T: 'static + Copy> BorrowMut<T> for Map<T> {
    fn borrow_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

impl<T: 'static + Copy, I: SliceIndex<[u8]>> Index<I> for Map<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        let slice = unsafe { from_raw_parts(self.0 as *const u8, self.1) };
        Index::index(&slice[size_of::<T>()..], index)
    }
}

impl<T: 'static + Copy, I: SliceIndex<[u8]>> IndexMut<I> for Map<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let slice = unsafe { from_raw_parts_mut(self.0 as *mut u8, self.1) };
        IndexMut::index_mut(&mut slice[size_of::<T>()..], index)
    }
}

impl<T: 'static + Copy> Map<T> {
    #[inline]
    pub fn build(access: Access) -> Builder<T> {
        Builder {
            phantom: PhantomData,
            access,
            offset: 0,
            extra: 0,
            flags: Flags::default(),
            prot: Protection::default(),
            addr: 0, // NULL
            fd: -1,
        }
    }

    pub unsafe fn cast<U: 'static + Copy>(self) -> Map<U> {
        let map = Map(self.0 as *mut U, self.1);
        std::mem::forget(self);
        map
    }
}

impl<T: 'static + Copy> Builder<T> {
    #[inline]
    pub fn protection(mut self, prot: Protection) -> Self {
        self.prot = prot;
        self
    }

    #[inline]
    pub fn extra(mut self, extra: usize) -> Self {
        self.extra = extra;
        self
    }

    #[inline]
    pub fn flags(mut self, flags: Flags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub fn address(mut self, addr: libc::uintptr_t) -> Self {
        self.addr = addr;
        self
    }

    #[inline]
    pub fn file(mut self, fd: &impl AsRawFd, offset: libc::off_t) -> Self {
        self.fd = fd.as_raw_fd();
        self.offset = offset;
        self
    }

    #[inline]
    pub fn done(self) -> Result<Map<T>> {
        let length = size_of::<T>() + self.extra;

        let access = match self.access {
            Access::Private => libc::MAP_PRIVATE,
            Access::Shared => libc::MAP_SHARED,
        };

        let ptr = unsafe {
            libc::mmap(
                self.addr as *mut c_void,
                length,
                self.prot.bits(),
                access | self.flags.bits(),
                self.fd,
                self.offset,
            )
        };

        if ptr.is_null() {
            Err(Error::last_os_error())?
        }

        Ok(Map(ptr as *mut T, length))
    }
}
