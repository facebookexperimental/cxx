#![allow(missing_docs)]
#![allow(clippy::let_unit_value)]
#![allow(clippy::ref_option_ref)]

use core::mem;

#[cfg(feature = "alloc")]
use crate::private::RustString;
#[cfg(feature = "alloc")]
use crate::private::RustVec;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::pin::Pin;

mod private {
    pub trait Sealed {}
}
pub trait OptionTarget: private::Sealed {}

impl<T: Sized> private::Sealed for &T {}
impl<T: Sized> OptionTarget for &T {}

impl<T: Sized> private::Sealed for &mut T {}
impl<T: Sized> OptionTarget for &mut T {}

impl<T: Sized> private::Sealed for Pin<&mut T> {}
impl<T: Sized> OptionTarget for Pin<&mut T> {}

#[cfg(feature = "alloc")]
impl<T: Sized> private::Sealed for Box<T> {}
#[cfg(feature = "alloc")]
impl<T: Sized> OptionTarget for Box<T> {}

type Repr =
    [mem::MaybeUninit<usize>; mem::size_of::<Option<&()>>() / core::mem::size_of::<usize>()];

// ABI compatible with C++ rust::Option<T> (not necessarily core::option::Option<T>).
#[repr(C)]
pub struct RustOption<T: OptionTarget> {
    repr: Repr,
    marker: core::marker::PhantomData<T>,
}

pub const fn assert_option_safe<T>() {
    struct __SizeCheck<U>(core::marker::PhantomData<U>);
    impl<U> __SizeCheck<U> {
        const _IS_OPTION_SIZE: () = assert!(mem::size_of::<Option<U>>() == mem::size_of::<Repr>());
        const _IS_USIZE: () = assert!(mem::size_of::<Repr>() == mem::size_of::<usize>());
        const _IS_NICHE: () = assert!(mem::size_of::<Option<U>>() == mem::size_of::<U>());
        const _IS_USIZE_ALIGN: () = assert!(mem::align_of::<Repr>() == mem::align_of::<usize>());
        const _IS_OPTION_ALIGN: () =
            assert!(mem::align_of::<Option<U>>() == mem::align_of::<Repr>());
    }
    // Force the constants to resolve (at compile time)
    let _: () = __SizeCheck::<T>::_IS_OPTION_SIZE;
    let _: () = __SizeCheck::<T>::_IS_USIZE;
    let _: () = __SizeCheck::<T>::_IS_NICHE;
    let _: () = __SizeCheck::<T>::_IS_USIZE_ALIGN;
    let _: () = __SizeCheck::<T>::_IS_OPTION_ALIGN;
}

impl<T: OptionTarget> RustOption<T> {
    pub fn new() -> Self {
        let _: () = assert_option_safe::<T>();
        Self::from(None)
    }

    pub fn into_option(mut self) -> Option<T> {
        let _: () = assert_option_safe::<T>();
        self.as_mut_option().take()
    }

    pub fn as_option(&self) -> &Option<T> {
        let _: () = assert_option_safe::<T>();
        unsafe { &*(self as *const RustOption<T> as *const Option<T>) }
    }

    pub fn as_mut_option(&mut self) -> &mut Option<T> {
        let _: () = assert_option_safe::<T>();
        unsafe { &mut *(self as *mut RustOption<T> as *mut Option<T>) }
    }

    pub fn from(o: Option<T>) -> Self {
        let _: () = assert_option_safe::<T>();
        let v = unsafe { core::mem::transmute_copy(&o) };
        core::mem::forget(o);
        v
    }

    pub fn from_ref(o: &Option<T>) -> &Self {
        let _: () = assert_option_safe::<T>();
        unsafe { &*(o as *const Option<T> as *const RustOption<T>) }
    }

    pub fn from_mut(o: &mut Option<T>) -> &mut Self {
        let _: () = assert_option_safe::<T>();
        unsafe { &mut *(o as *mut Option<T> as *mut RustOption<T>) }
    }

    pub fn value(&self) -> Option<&T> {
        self.as_option().as_ref()
    }

    pub fn has_value(&self) -> bool {
        self.as_option().is_some()
    }

    pub fn set(&mut self, value: T) {
        self.as_mut_option().replace(value);
    }

    pub unsafe fn as_ref_mut_inner_unchecked(&mut self) -> &mut T {
        unsafe { self.as_mut_option().as_mut().unwrap_unchecked() }
    }
}

impl<'a, T> RustOption<&'a T> {
    pub fn into_raw(self) -> *const T {
        self.into_option()
            .map_or(core::ptr::null(), |v| v as *const T)
    }

    pub fn into_raw_improper(self) -> *const core::ffi::c_void {
        self.into_option().map_or(core::ptr::null(), |v| {
            v as *const T as *const core::ffi::c_void
        })
    }

    /// SAFETY: ptr must be valid for 'a
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        let mut this = RustOption::new();
        if let Some(r) = unsafe { ptr.as_ref() } {
            this.set(r);
        }
        this
    }

    /// SAFETY: ptr must be valid for 'a, and castable to *const T
    pub unsafe fn from_raw_improper(ptr: *const core::ffi::c_void) -> Self {
        let mut this = RustOption::new();
        let ptr = ptr as *const T;
        if let Some(r) = unsafe { ptr.as_ref() } {
            this.set(r);
        }
        this
    }
}

impl<'a, T> RustOption<&'a mut T> {
    pub fn into_raw(self) -> *mut T {
        self.into_option()
            .map_or(core::ptr::null_mut(), |v| v as *mut T)
    }

    pub fn into_raw_improper(self) -> *mut core::ffi::c_void {
        self.into_option().map_or(core::ptr::null_mut(), |v| {
            v as *mut T as *mut core::ffi::c_void
        })
    }

    /// SAFETY: ptr must be valid for 'a
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        let mut this = RustOption::new();
        if let Some(r) = unsafe { ptr.as_mut() } {
            this.set(r);
        }
        this
    }

    /// SAFETY: ptr must be valid for 'a, and castable to *mut T
    pub unsafe fn from_raw_improper(ptr: *mut core::ffi::c_void) -> Self {
        let mut this = RustOption::new();
        let ptr = ptr as *mut T;
        if let Some(r) = unsafe { ptr.as_mut() } {
            this.set(r);
        }
        this
    }
}

impl<'a, T> RustOption<Pin<&'a mut T>> {
    pub fn into_raw(self) -> *mut T {
        self.into_option()
            .map_or(core::ptr::null_mut(), |v| unsafe {
                v.get_unchecked_mut() as *mut T
            })
    }

    pub fn into_raw_improper(self) -> *mut core::ffi::c_void {
        self.into_option()
            .map_or(core::ptr::null_mut(), |v| unsafe {
                v.get_unchecked_mut() as *mut T as *mut core::ffi::c_void
            })
    }

    /// SAFETY: ptr must be valid for 'a
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        let mut this = RustOption::new();
        if let Some(r) = unsafe { ptr.as_mut() } {
            this.set(unsafe { Pin::new_unchecked(r) });
        }
        this
    }

    /// SAFETY: ptr must be valid for 'a, and castable to *mut T
    pub unsafe fn from_raw_improper(ptr: *mut core::ffi::c_void) -> Self {
        let mut this = RustOption::new();
        let ptr = ptr as *mut T;
        if let Some(r) = unsafe { ptr.as_mut() } {
            this.set(unsafe { Pin::new_unchecked(r) });
        }
        this
    }
}

#[cfg(feature = "alloc")]
impl<T> RustOption<Box<T>> {
    pub fn into_raw(self) -> *mut T {
        self.into_option()
            .map_or(core::ptr::null_mut(), |v| Box::into_raw(v))
    }

    pub fn into_raw_improper(self) -> *mut core::ffi::c_void {
        self.into_option().map_or(core::ptr::null_mut(), |v| {
            Box::into_raw(v) as *mut core::ffi::c_void
        })
    }

    /// SAFETY: ptr must have originated from a `Option<Box<T>>`
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        let mut this = RustOption::new();
        if !ptr.is_null() {
            this.set(unsafe { Box::from_raw(ptr) });
        }
        this
    }

    /// SAFETY: ptr must have originated from a `Option<Box<T>>`
    pub unsafe fn from_raw_improper(ptr: *mut core::ffi::c_void) -> Self {
        let mut this = RustOption::new();
        if !ptr.is_null() {
            this.set(unsafe { Box::from_raw(ptr as *mut T) });
        }
        this
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> RustOption<&'a RustVec<T>> {
    pub fn from_option_vec_ref(other: Option<&'a Vec<T>>) -> Self {
        unsafe {
            core::mem::transmute::<RustOption<&Vec<T>>, RustOption<&RustVec<T>>>(RustOption::from(
                other,
            ))
        }
    }

    pub fn into_option_vec_ref(self) -> Option<&'a Vec<T>> {
        unsafe { core::mem::transmute::<RustOption<&RustVec<T>>, RustOption<&Vec<T>>>(self) }
            .into_option()
    }

    pub fn as_option_vec_ref(&self) -> &Option<&'a Vec<T>> {
        unsafe { &*(self as *const RustOption<&RustVec<T>> as *const RustOption<&Vec<T>>) }
            .as_option()
    }

    pub fn as_option_vec_ref_mut(&mut self) -> &mut Option<&'a Vec<T>> {
        unsafe { &mut *(self as *mut RustOption<&RustVec<T>> as *mut RustOption<&Vec<T>>) }
            .as_mut_option()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> RustOption<&'a mut RustVec<T>> {
    pub fn from_option_vec_mut(other: Option<&'a mut Vec<T>>) -> Self {
        unsafe {
            core::mem::transmute::<RustOption<&mut Vec<T>>, RustOption<&mut RustVec<T>>>(
                RustOption::from(other),
            )
        }
    }

    pub fn into_option_vec_mut(self) -> Option<&'a mut Vec<T>> {
        unsafe {
            core::mem::transmute::<RustOption<&mut RustVec<T>>, RustOption<&mut Vec<T>>>(self)
        }
        .into_option()
    }

    pub fn as_option_vec_mut(&self) -> &Option<&'a mut Vec<T>> {
        unsafe {
            &*(self as *const RustOption<&'a mut RustVec<T>> as *const RustOption<&'a mut Vec<T>>)
        }
        .as_option()
    }

    pub fn as_option_vec_mut_mut(&mut self) -> &mut Option<&'a mut Vec<T>> {
        unsafe {
            &mut *(self as *mut RustOption<&'a mut RustVec<T>> as *mut RustOption<&'a mut Vec<T>>)
        }
        .as_mut_option()
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a RustVec<RustString>> {
    pub fn from_option_vec_string_ref(other: Option<&'a Vec<String>>) -> Self {
        unsafe {
            core::mem::transmute::<RustOption<&Vec<String>>, RustOption<&RustVec<RustString>>>(
                RustOption::from(other),
            )
        }
    }

    pub fn into_option_vec_string_ref(self) -> Option<&'a Vec<String>> {
        unsafe {
            core::mem::transmute::<RustOption<&RustVec<RustString>>, RustOption<&Vec<String>>>(self)
        }
        .into_option()
    }

    pub fn as_option_vec_string_ref_mut(&mut self) -> &mut Option<&'a Vec<String>> {
        unsafe {
            &mut *(self as *mut RustOption<&RustVec<RustString>> as *mut RustOption<&Vec<String>>)
        }
        .as_mut_option()
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a mut RustVec<RustString>> {
    pub fn from_option_vec_string_mut(other: Option<&'a mut Vec<String>>) -> Self {
        unsafe {
            core::mem::transmute::<RustOption<&mut Vec<String>>, RustOption<&mut RustVec<RustString>>>(
                RustOption::from(other),
            )
        }
    }

    pub fn into_option_vec_string_mut(self) -> Option<&'a mut Vec<String>> {
        unsafe { core::mem::transmute::<RustOption<&mut RustVec<RustString>>, RustOption<&mut Vec<String>>>(self) }.into_option()
    }

    pub fn as_option_vec_string_mut_mut(&mut self) -> &mut Option<&'a mut Vec<String>> {
        unsafe {
            (*(self as *mut RustOption<&mut RustVec<RustString>> as *mut RustOption<&mut Vec<alloc::string::String>>))
            .as_mut_option()
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a RustString> {
    pub fn from_option_string_ref(other: Option<&'a String>) -> Self {
        unsafe {
            core::mem::transmute::<RustOption<&String>, RustOption<&RustString>>(RustOption::from(
                other,
            ))
        }
    }

    pub fn into_option_string_ref(self) -> Option<&'a String> {
        unsafe { core::mem::transmute::<RustOption<&RustString>, RustOption<&String>>(self) }
            .into_option()
    }

    pub fn as_option_string_ref_mut(&mut self) -> &mut Option<&'a String> {
        unsafe { &mut *(self as *mut RustOption<&RustString> as *mut RustOption<&String>) }
            .as_mut_option()
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a mut RustString> {
    pub fn from_option_string_mut(other: Option<&'a mut String>) -> Self {
        unsafe {
            core::mem::transmute::<RustOption<&mut String>, RustOption<&mut RustString>>(
                RustOption::from(other),
            )
        }
    }

    pub fn into_option_string_mut(self) -> Option<&'a mut String> {
        unsafe {
            core::mem::transmute::<RustOption<&mut RustString>, RustOption<&mut String>>(self)
        }
        .into_option()
    }

    pub fn as_option_string_mut_mut(&mut self) -> &mut Option<&'a mut String> {
        unsafe {
            (*(self as *mut RustOption<&mut RustString> as *mut RustOption<&mut String>))
            .as_mut_option()
        }
    }
}

impl<T: OptionTarget> Drop for RustOption<T> {
    fn drop(&mut self) {
        self.as_mut_option().take();
    }
}
