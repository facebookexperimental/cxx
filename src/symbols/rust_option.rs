use crate::rust_option::OptionFfi;
use crate::rust_option::OptionFfiInverse;
#[cfg(feature = "alloc")]
use crate::rust_string::RustString;
#[cfg(feature = "alloc")]
use crate::rust_vec::RustVec;
use core::ffi::c_char;
use core::mem;
use core::ptr;

macro_rules! rust_option_shims {
    ($segment:expr, $ty:ty) => {
        const_assert_eq!(
            mem::size_of::<Option<&$ty>>(),
            mem::size_of::<<Option<&$ty> as OptionFfi>::Target>()
        );
        const_assert_eq!(mem::size_of::<Option<&$ty>>(), mem::size_of::<usize>());

        const _: () = {
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$new")]
            unsafe extern "C" fn __const_new(this: *mut <Option<&$ty> as OptionFfi>::Target) {
                unsafe { ptr::write(this, Option::<&$ty>::new_ffi()) };
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$drop")]
            unsafe extern "C" fn __const_drop(this: *mut <Option<&$ty> as OptionFfi>::Target) {
                unsafe { ptr::drop_in_place(this) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$has_value")]
            unsafe extern "C" fn __const_has_value(this: *const <Option<&$ty> as OptionFfi>::Target) -> bool {
                let o: &<Option<&$ty> as OptionFfi>::Target = unsafe { this.as_ref().unwrap() };
                o.as_option().is_some()
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$value")]
            unsafe extern "C" fn __const_value(this: *const <Option<&$ty> as OptionFfi>::Target) -> *const $ty {
                unsafe { this.as_ref().unwrap().as_option().as_ref().copied().unwrap() as *const $ty }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$set")]
            unsafe extern "C" fn __const_set(
                this: *mut <Option<&$ty> as OptionFfi>::Target,
                value: *mut $ty,
            ) {
                unsafe { this.as_mut().unwrap().set(&*value) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$new")]
            unsafe extern "C" fn __new(this: *mut <Option<&mut $ty> as OptionFfi>::Target) {
                unsafe { ptr::write(this, Option::<&mut $ty>::new_ffi()) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$drop")]
            unsafe extern "C" fn __drop(this: *mut <Option<&mut $ty> as OptionFfi>::Target) {
                unsafe { ptr::drop_in_place(this) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$has_value")]
            unsafe extern "C" fn __has_value(this: *const <Option<&mut $ty> as OptionFfi>::Target) -> bool {
                let o: &<Option<&mut $ty> as OptionFfi>::Target = unsafe { this.as_ref().unwrap() };
                o.as_option().is_some()
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$value_const")]
            unsafe extern "C" fn __value_const(this: *const <Option<&mut $ty> as OptionFfi>::Target) -> *const $ty {
                let v: &$ty = unsafe { this.as_ref().unwrap().as_option().as_ref().unwrap() };
                v as *const $ty
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$value")]
            unsafe extern "C" fn __value(this: *mut <Option<&mut $ty> as OptionFfi>::Target) -> *mut $ty {
                let this = unsafe { this.as_mut().unwrap() };
                let ptr = this.as_mut_option().as_mut().unwrap();
                *ptr as _
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$set")]
            unsafe extern "C" fn __set(
                this: *mut <Option<&mut $ty> as OptionFfi>::Target,
                value: *mut $ty,
            ) {
                unsafe { this.as_mut().unwrap().set(&mut *value) }
            }
        };
        #[cfg(feature = "alloc")]
        const _: () = {
            /* Vec<T> impl */
            #[export_name = concat!("cxxbridge1$rust_option$const$rust_vec$", $segment, "$new")]
            unsafe extern "C" fn __const_new(this: *mut <Option<&RustVec<$ty>> as OptionFfi>::Target) {
                unsafe { ptr::write(this, Option::<&RustVec<$ty>>::new_ffi()) };
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$rust_vec$", $segment, "$drop")]
            unsafe extern "C" fn __const_drop(this: *mut <Option<&RustVec<$ty>> as OptionFfi>::Target) {
                unsafe { ptr::drop_in_place(this) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$rust_vec$", $segment, "$has_value")]
            unsafe extern "C" fn __const_has_value(this: *const <Option<&RustVec<$ty>> as OptionFfi>::Target) -> bool {
                let o: &<Option<&RustVec<$ty>> as OptionFfi>::Target = unsafe { this.as_ref().unwrap() };
                <Option<&alloc::vec::Vec<$ty>> as OptionFfi>::Target::as_option_vec_ref(o).is_some()
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$rust_vec$", $segment, "$value")]
            unsafe extern "C" fn __const_value(this: *const <Option<&RustVec<$ty>> as OptionFfi>::Target) -> *const RustVec<$ty> {
                unsafe { this.as_ref().unwrap().as_option().as_ref().copied().unwrap() as *const RustVec<$ty> }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$rust_vec$", $segment, "$set")]
            unsafe extern "C" fn __const_set(
                this: *mut <Option<&RustVec<$ty>> as OptionFfi>::Target,
                value: *mut RustVec<$ty>,
            ) {
                unsafe { <Option<&alloc::vec::Vec<$ty>> as OptionFfi>::Target::as_option_vec_ref_mut(this.as_mut().unwrap()).replace((&*value).as_vec()); }

            }
            #[export_name = concat!("cxxbridge1$rust_option$rust_vec$", $segment, "$new")]
            unsafe extern "C" fn __new(this: *mut <Option<&mut RustVec<$ty>> as OptionFfi>::Target) {
                unsafe { ptr::write(this, Option::<&mut RustVec<$ty>>::new_ffi()) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$rust_vec$", $segment, "$drop")]
            unsafe extern "C" fn __drop(this: *mut <Option<&mut RustVec<$ty>> as OptionFfi>::Target) {
                unsafe { ptr::drop_in_place(this) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$rust_vec$", $segment, "$has_value")]
            unsafe extern "C" fn __has_value(this: *const <Option<&mut RustVec<$ty>> as OptionFfi>::Target) -> bool {
                let o: &<Option<&mut RustVec<$ty>> as OptionFfi>::Target = unsafe { this.as_ref().unwrap() };
                <Option<&mut alloc::vec::Vec<$ty>> as OptionFfi>::Target::as_option_vec_mut(o).is_some()
            }
            #[export_name = concat!("cxxbridge1$rust_option$rust_vec$", $segment, "$value_const")]
            unsafe extern "C" fn __value_const(this: *const <Option<&mut RustVec<$ty>> as OptionFfi>::Target) -> *const RustVec<$ty> {
                let v: &alloc::vec::Vec<_> = unsafe { <Option<&mut alloc::vec::Vec<$ty>> as OptionFfi>::Target::as_option_vec_mut(this.as_ref().unwrap()).as_ref().unwrap() };
                v as *const alloc::vec::Vec<$ty> as *const RustVec<$ty>
            }
            #[export_name = concat!("cxxbridge1$rust_option$rust_vec$", $segment, "$value")]
            unsafe extern "C" fn __value(this: *mut <Option<&mut RustVec<$ty>> as OptionFfi>::Target) -> *mut RustVec<$ty> {
                let ptr = unsafe { <Option<&mut alloc::vec::Vec<$ty>> as OptionFfi>::Target::as_option_vec_mut_mut(this.as_mut().unwrap()).as_mut().unwrap() };
                *ptr as *mut alloc::vec::Vec<$ty> as *mut RustVec<$ty>
            }
            #[export_name = concat!("cxxbridge1$rust_option$rust_vec$", $segment, "$set")]
            unsafe extern "C" fn __set(
                this: *mut <Option<&mut RustVec<$ty>> as OptionFfi>::Target,
                value: *mut RustVec<$ty>,
            ) {
                unsafe { <Option<&mut alloc::vec::Vec<$ty>> as OptionFfi>::Target::as_option_vec_mut_mut(this.as_mut().unwrap()).replace((&mut *value).as_mut_vec()); }
            }
        };
    };
}

macro_rules! rust_option_shims_for_primitive {
    ($ty:ident) => {
        rust_option_shims!(stringify!($ty), $ty);
    };
}

rust_option_shims_for_primitive!(u8);
rust_option_shims_for_primitive!(bool);
rust_option_shims_for_primitive!(u16);
rust_option_shims_for_primitive!(u32);
rust_option_shims_for_primitive!(u64);
rust_option_shims_for_primitive!(usize);
rust_option_shims_for_primitive!(i8);
rust_option_shims_for_primitive!(i16);
rust_option_shims_for_primitive!(i32);
rust_option_shims_for_primitive!(i64);
rust_option_shims_for_primitive!(isize);
rust_option_shims_for_primitive!(f32);
rust_option_shims_for_primitive!(f64);

rust_option_shims!("char", c_char);
#[cfg(feature = "alloc")]
rust_option_shims!("string", RustString);
