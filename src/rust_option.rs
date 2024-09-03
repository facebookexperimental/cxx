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
pub trait OptionPtrTarget: private::Sealed {}

impl<T: Sized> private::Sealed for &T {}
impl<T: Sized> OptionPtrTarget for &T {}

impl<T: Sized> private::Sealed for &mut T {}
impl<T: Sized> OptionPtrTarget for &mut T {}

impl<T: Sized> private::Sealed for Pin<&mut T> {}
impl<T: Sized> OptionPtrTarget for Pin<&mut T> {}

#[cfg(feature = "alloc")]
impl<T: Sized> private::Sealed for Box<T> {}
#[cfg(feature = "alloc")]
impl<T: Sized> OptionPtrTarget for Box<T> {}

pub trait OptionFfi: private::Sealed {
    // The FFI type that this Rust type is mapped to
    type Target;

    // Option -> RustOption
    fn new_ffi() -> Self::Target;
    fn into_ffi(self) -> Self::Target;
    fn as_ffi(&self) -> &Self::Target;
    fn as_mut_ffi(&mut self) -> &mut Self::Target;
    // RustOption -> Option
    fn from_ffi(other: Self::Target) -> Self;
    fn from_ffi_ref(other: &Self::Target) -> &Self;
    fn from_ffi_mut(other: &mut Self::Target) -> &mut Self;
}

// Trait will be implemented by RustOption<T> for some Ts
pub trait OptionFfiInverse: private::Sealed + Sized {
    // The Rust type that this FFI type is mapped to
    type Target: OptionFfi<Target = Self>;

    fn new() -> Self {
        <Self::Target as OptionFfi>::new_ffi()
    }

    // RustOption -> Option
    fn into_option(self) -> Self::Target {
        <Self::Target as OptionFfi>::from_ffi(self)
    }

    fn as_option(&self) -> &Self::Target {
        <Self::Target as OptionFfi>::from_ffi_ref(self)
    }

    fn as_mut_option(&mut self) -> &mut Self::Target {
        <Self::Target as OptionFfi>::from_ffi_mut(self)
    }

    // Option -> RustOption
    fn from_option(other: Self::Target) -> Self {
        <Self::Target as OptionFfi>::into_ffi(other)
    }

    fn from_option_ref(other: &Self::Target) -> &Self {
        <Self::Target as OptionFfi>::as_ffi(other)
    }

    fn from_option_mut(other: &mut Self::Target) -> &mut Self {
        <Self::Target as OptionFfi>::as_mut_ffi(other)
    }
}

/// Defined a struct named RustOption and implements OptionFfi for Option with it as target
macro_rules! impl_option_ffi {
    // Like `impl<T: Bound> RustOption<T>` where you need some bound on T
    (<$generic:ident: $bound:path>, $repr:ty, $sizing:ty) => {
        impl_option_ffi!(_private: generics=<>, bounded_generics=<$generic: $bound>, option_ty=$generic, repr=$repr, sizing=$sizing)
    };
    // Like `impl<T> RustOption<S<T>>` for some concrete S and generic T
    (<$generic:ident>, $t:ty, $repr:ty, $sizing:ty) => {
        impl_option_ffi!(_private: generics=<$generic>, bounded_generics=<>, option_ty=$t, repr=$repr, sizing=$sizing)
    };
    // Like `impl RustOption<T>` for some non-generic T
    (<$t:ident>, $repr:ty, $sizing:ty) => {
        impl_option_ffi!(_private: generics=<>, bounded_generics=<>, option_ty=$t, repr=$repr, sizing=$sizing)
    };
    // Private case. Does the actual implementation
    (_private: generics=<$($generic1:ident),*>, bounded_generics=<$($generic2:ident: $bound:path),*>, option_ty=$option_ty:ty, repr=$repr:ty, sizing=$sizing:ty) => {
        type Repr = [mem::MaybeUninit<$repr>; mem::size_of::<Option<$sizing>>() / mem::size_of::<$repr>()];

        // ABI compatible with C++ rust::Option<T> for (not necessarily core::option::Option<T>).
        pub struct RustOption<$($generic1),* $($generic2: $bound),*> {
            #[allow(dead_code)]
            repr: Repr,
            phantom: core::marker::PhantomData<Option<$option_ty>>,
        }

        pub const fn assert_option_safe<T>() {
            struct __SizeCheck<U>(core::marker::PhantomData<U>);
            impl<U> __SizeCheck<U> {
                const _IS_OPTION_SIZE: () =
                    assert!(mem::size_of::<Option<U>>() == mem::size_of::<Repr>());
                const _IS_REPR_ALIGN: () =
                    assert!(mem::align_of::<Repr>() == mem::align_of::<$repr>());
                const _IS_OPTION_ALIGN: () =
                    assert!(mem::align_of::<Option<U>>() == mem::align_of::<Repr>());
            }
            // Force the constants to resolve (at compile time)
            let _: () = __SizeCheck::<T>::_IS_OPTION_SIZE;
            let _: () = __SizeCheck::<T>::_IS_REPR_ALIGN;
            let _: () = __SizeCheck::<T>::_IS_OPTION_ALIGN;
        }

        impl<$($generic1),* $($generic2: $bound),*> private::Sealed for Option<$option_ty> {}

        impl<$($generic1),* $($generic2: $bound),*> OptionFfi for Option<$option_ty> {
            type Target = RustOption<$($generic1),* $($generic2),*>;

            fn new_ffi() -> Self::Target {
                Self::None.into_ffi()
            }

            fn into_ffi(self) -> Self::Target {
                let _: () = assert_option_safe::<$option_ty>();
                let v = unsafe { core::mem::transmute_copy(&self) };
                core::mem::forget(self);
                v
            }

            fn as_ffi(&self) -> &Self::Target {
                let _: () = assert_option_safe::<$option_ty>();
                unsafe { &*(self as *const Self as *const Self::Target) }
            }

            fn as_mut_ffi(&mut self) -> &mut Self::Target {
                let _: () = assert_option_safe::<$option_ty>();
                unsafe { &mut *(self as *mut Self as *mut Self::Target) }
            }

            fn from_ffi(mut other: Self::Target) -> Self {
                let _: () = assert_option_safe::<$option_ty>();
                Self::from_ffi_mut(&mut other).take()
            }

            fn from_ffi_ref(other: &Self::Target) -> &Self {
                let _: () = assert_option_safe::<$option_ty>();
                unsafe { &*(other as *const Self::Target as *const Self) }
            }

            fn from_ffi_mut(other: &mut Self::Target) -> &mut Self {
                let _: () = assert_option_safe::<$option_ty>();
                unsafe { &mut *(other as *mut Self::Target as *mut Self) }
            }
        }

        impl<$($generic1),* $($generic2: $bound),*> private::Sealed for RustOption<$($generic1),* $($generic2),*> {}

        impl<$($generic1),* $($generic2: $bound),*> OptionFfiInverse for RustOption<$($generic1),* $($generic2),*> {
            type Target = Option<$option_ty>;
        }

        impl<$($generic1),* $($generic2: $bound),*> Drop for RustOption<$($generic1),* $($generic2),*> {
            fn drop(&mut self) {
                self.as_mut_option().take();
            }
        }
    };
}

// Pointer-sized pointer types with niche optimization
const _: () = {
    impl_option_ffi! { <T: OptionPtrTarget>, usize, &'static () }

    impl<T: OptionPtrTarget> RustOption<T> {
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
    impl<'a, T> RustOption<&'a Vec<T>> {
        pub fn from_option_vec_ref(other: Option<&'a Vec<T>>) -> RustOption<&'a RustVec<T>> {
            unsafe {
                core::mem::transmute::<RustOption<&Vec<T>>, RustOption<&RustVec<T>>>(
                    RustOption::from_option(other),
                )
            }
        }

        pub fn into_option_vec_ref(this: RustOption<&'a RustVec<T>>) -> Option<&'a Vec<T>> {
            unsafe { core::mem::transmute::<RustOption<&RustVec<T>>, RustOption<&Vec<T>>>(this) }
                .into_option()
        }

        pub fn as_option_vec_ref<'b>(
            this: &'b RustOption<&'a RustVec<T>>,
        ) -> &'b Option<&'a Vec<T>> {
            unsafe { &*(this as *const RustOption<&RustVec<T>> as *const RustOption<&Vec<T>>) }
                .as_option()
        }

        pub fn as_option_vec_ref_mut<'b>(
            this: &'b mut RustOption<&'a RustVec<T>>,
        ) -> &'b mut Option<&'a Vec<T>> {
            unsafe { &mut *(this as *mut RustOption<&RustVec<T>> as *mut RustOption<&Vec<T>>) }
                .as_mut_option()
        }

        pub fn into_rust_option_rust_vec_ref(self) -> RustOption<&'a RustVec<T>> {
            unsafe { core::mem::transmute::<RustOption<&Vec<T>>, RustOption<&RustVec<T>>>(self) }
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a, T> RustOption<&'a mut Vec<T>> {
        pub fn from_option_vec_mut(
            other: Option<&'a mut Vec<T>>,
        ) -> RustOption<&'a mut RustVec<T>> {
            unsafe {
                core::mem::transmute::<RustOption<&mut Vec<T>>, RustOption<&mut RustVec<T>>>(
                    RustOption::from_option(other),
                )
            }
        }

        pub fn into_option_vec_mut(this: RustOption<&'a mut RustVec<T>>) -> Option<&'a mut Vec<T>> {
            unsafe {
                core::mem::transmute::<RustOption<&mut RustVec<T>>, RustOption<&mut Vec<T>>>(this)
            }
            .into_option()
        }

        pub fn as_option_vec_mut<'b>(
            this: &'b RustOption<&'a mut RustVec<T>>,
        ) -> &'b Option<&'a mut Vec<T>> {
            unsafe {
                &*(this as *const RustOption<&'a mut RustVec<T>>
                    as *const RustOption<&'a mut Vec<T>>)
            }
            .as_option()
        }

        pub fn as_option_vec_mut_mut<'b>(
            this: &'b mut RustOption<&'a mut RustVec<T>>,
        ) -> &'b mut Option<&'a mut Vec<T>> {
            unsafe {
                &mut *(this as *mut RustOption<&'a mut RustVec<T>>
                    as *mut RustOption<&'a mut Vec<T>>)
            }
            .as_mut_option()
        }

        pub fn into_rust_option_rust_vec_mut(self) -> RustOption<&'a mut RustVec<T>> {
            unsafe {
                core::mem::transmute::<RustOption<&mut Vec<T>>, RustOption<&mut RustVec<T>>>(self)
            }
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> RustOption<&'a Vec<String>> {
        pub fn from_option_vec_string_ref(
            other: Option<&'a Vec<String>>,
        ) -> RustOption<&'a RustVec<RustString>> {
            unsafe {
                core::mem::transmute::<RustOption<&Vec<String>>, RustOption<&RustVec<RustString>>>(
                    RustOption::from_option(other),
                )
            }
        }

        pub fn into_option_vec_string_ref(
            this: RustOption<&'a RustVec<RustString>>,
        ) -> Option<&'a Vec<String>> {
            unsafe {
                core::mem::transmute::<RustOption<&RustVec<RustString>>, RustOption<&Vec<String>>>(
                    this,
                )
            }
            .into_option()
        }

        pub fn as_option_vec_string_ref_mut<'b>(
            this: &'b mut RustOption<&'a RustVec<RustString>>,
        ) -> &'b mut Option<&'a Vec<String>> {
            unsafe {
                &mut *(this as *mut RustOption<&RustVec<RustString>>
                    as *mut RustOption<&Vec<String>>)
            }
            .as_mut_option()
        }

        pub fn into_rust_option_rust_vec_rust_string_ref(
            self,
        ) -> RustOption<&'a RustVec<RustString>> {
            unsafe {
                core::mem::transmute::<RustOption<&Vec<String>>, RustOption<&RustVec<RustString>>>(
                    self,
                )
            }
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> RustOption<&'a mut Vec<String>> {
        pub fn from_option_vec_string_mut(
            other: Option<&'a mut Vec<String>>,
        ) -> RustOption<&'a mut RustVec<RustString>> {
            unsafe {
                core::mem::transmute::<
                    RustOption<&mut Vec<String>>,
                    RustOption<&mut RustVec<RustString>>,
                >(RustOption::from_option(other))
            }
        }

        pub fn into_option_vec_string_mut(
            this: RustOption<&'a mut RustVec<RustString>>,
        ) -> Option<&'a mut Vec<String>> {
            unsafe {
                core::mem::transmute::<
                    RustOption<&mut RustVec<RustString>>,
                    RustOption<&mut Vec<String>>,
                >(this)
            }
            .into_option()
        }

        pub fn as_option_vec_string_mut_mut<'b>(
            this: &'b mut RustOption<&'a mut RustVec<RustString>>,
        ) -> &'b mut Option<&'a mut Vec<String>> {
            unsafe {
                &mut *(this as *mut RustOption<&mut RustVec<RustString>> as *mut RustOption<&mut Vec<String>>)
            }.as_mut_option()
        }

        pub fn into_rust_option_rust_vec_rust_string_mut(
            self,
        ) -> RustOption<&'a mut RustVec<RustString>> {
            unsafe {
                core::mem::transmute::<
                    RustOption<&mut Vec<String>>,
                    RustOption<&mut RustVec<RustString>>,
                >(self)
            }
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> RustOption<&'a String> {
        pub fn from_option_string_ref(other: Option<&'a String>) -> RustOption<&'a RustString> {
            unsafe {
                core::mem::transmute::<RustOption<&String>, RustOption<&RustString>>(
                    RustOption::from_option(other),
                )
            }
        }

        pub fn into_option_string_ref(this: RustOption<&'a RustString>) -> Option<&'a String> {
            unsafe { core::mem::transmute::<RustOption<&RustString>, RustOption<&String>>(this) }
                .into_option()
        }

        pub fn as_option_string_ref_mut<'b>(
            this: &'b mut RustOption<&'a RustString>,
        ) -> &'b mut Option<&'a String> {
            unsafe { &mut *(this as *mut RustOption<&RustString> as *mut RustOption<&String>) }
                .as_mut_option()
        }

        pub fn into_rust_option_rust_string_ref(self) -> RustOption<&'a RustString> {
            unsafe { core::mem::transmute::<RustOption<&String>, RustOption<&RustString>>(self) }
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> RustOption<&'a mut String> {
        pub fn from_option_string_mut(
            other: Option<&'a mut String>,
        ) -> RustOption<&'a mut RustString> {
            unsafe {
                core::mem::transmute::<RustOption<&mut String>, RustOption<&mut RustString>>(
                    RustOption::from_option(other),
                )
            }
        }

        pub fn into_option_string_mut(
            this: RustOption<&'a mut RustString>,
        ) -> Option<&'a mut String> {
            unsafe {
                core::mem::transmute::<RustOption<&mut RustString>, RustOption<&mut String>>(this)
            }
            .into_option()
        }

        pub fn as_option_string_mut_mut<'b>(
            this: &'b mut RustOption<&'a mut RustString>,
        ) -> &'b mut Option<&'a mut String> {
            unsafe {
                &mut *(this as *mut RustOption<&mut RustString> as *mut RustOption<&mut String>)
            }.as_mut_option()
        }

        pub fn into_rust_option_rust_string_mut(self) -> RustOption<&'a mut RustString> {
            unsafe {
                core::mem::transmute::<RustOption<&mut String>, RustOption<&mut RustString>>(self)
            }
        }
    }
};
