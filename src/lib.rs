extern crate mashup;

use std::collections::HashMap;

use std::sync::RwLock;

use dart_sys as ffi;

use lazy_static::lazy_static;
use std::ffi::{CStr, CString};
use crate::dart_handle::{UnverifiedDartHandle, DartHandle};
use std::os::raw::c_char;

mod function;
pub mod dart_handle;
pub mod dart_native_arguments;
pub mod dart_cobject;
pub mod dart_types;

lazy_static! {
    static ref REGISTER: RwLock<FunctionRegister> = RwLock::new(FunctionRegister::default());
}

pub type NativeFunction = unsafe extern "C" fn(arguments: ffi::Dart_NativeArguments);

#[derive(Default)]
#[allow(dead_code)] //Usage of these variables is declared in external crates.
pub struct FunctionRegister {
    functions: HashMap<&'static CStr, NativeFunction>,
    function_names: HashMap<NativeFunction, &'static CStr>,
}

#[allow(dead_code)] //Usage of these functions is declared in external crates.
impl FunctionRegister {
    pub(crate) fn add_function(&mut self, function: NativeFunction, name: &str) {
        let name = CString::new(name).unwrap();
        let name = unsafe {
            CStr::from_bytes_with_nul_unchecked(&*Box::leak::<'static>(name.into_bytes_with_nul().into_boxed_slice()))
        };
        self.functions.insert(name, function);
        self.function_names.insert(function, name);
    }

    pub(crate) unsafe fn get_function(&self, name: *const c_char) -> ffi::Dart_NativeFunction {
        let name = CStr::from_ptr::<'static>(name);
        self.functions.get(name).cloned()
    }

    pub(crate) fn get_name_from_function(&self, function: ffi::Dart_NativeFunction) -> Option<&'static CStr> {
        function.and_then(|x| self.function_names.get(&x)).map(|x| *x)
    }
}

#[allow(dead_code)] //Usage of this field is declared in external crates.
pub struct Registerer {
    pub(crate) export_fn: fn(&mut FunctionRegister),
}

#[allow(dead_code)] //Usage of this function is declared in external crates.
unsafe fn init(parent_library: ffi::Dart_Handle, registers: &[Registerer]) -> ffi::Dart_Handle {
    let parent_library = UnverifiedDartHandle::new(parent_library).get_error();
    if parent_library.is_err() {
        return parent_library.handle();
    }

    let mut lock = REGISTER.write().unwrap();
    for register in registers {
        (register.export_fn)(&mut *lock);
    }
    drop(lock);

    let result_code =
        ffi::Dart_SetNativeResolver(
            parent_library.handle(),
            Some(resolve_name),
            Some(resolve_function),
        );

    UnverifiedDartHandle::new(result_code)
        .get_error()
        .map(|_| UnverifiedDartHandle::null())
        .handle()
}

#[allow(dead_code)] //Usage of this function is declared in external crates.
unsafe extern "C" fn resolve_name(name: ffi::Dart_Handle, _argc: std::os::raw::c_int, _auto_scope_setup: *mut bool) -> ffi::Dart_NativeFunction {
    let name = UnverifiedDartHandle::new(name).get_error().ok()?;

    if !name.is_string() {
        return None;
    }

    let cname = dart_unwrap!(name.to_string());

    REGISTER.read().unwrap().get_function(cname.as_ptr())
}

unsafe extern "C" fn resolve_function(function: ffi::Dart_NativeFunction) -> *const u8 {
    let name = REGISTER.read().unwrap().get_name_from_function(function);
    if let Some(x) = name {
        x.as_ptr() as *const u8
    } else {
        b"<Unknown Native Function>\0" as &[u8] as *const [u8] as *const u8
    }
}

#[macro_export]
macro_rules! export_dart_functions {
    ($export_name: ident: $([$name:literal -> $function:ident]),*) => {
        static $ident: $crate::Registerer = Registerer {
            export_fn: {
                fn register_all(register: &mut $crate::FunctionRegister) {
                    $(
                        register.add_function($function, $name);
                    )*
                }
                register_all
            }
        };
    }
}

#[macro_export]
macro_rules! create_init_function {
    ($crate_name:ident, [$($name:ident),*$(,)?]) => {
        {
            mashup! {
                name["init"] = $crate_name _Init;
            }
            name! {
                #[allow(non_snake_case, unused_variables)]
                #[no_mangle]
                unsafe extern "C" fn "init"(parent_library: ffi::Dart_Handle) -> ffi::Dart_Handle {
                    $crate::init(parent_library, [$($name),*]);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! dart_unwrap {
    ($x: expr) => {
        match $x {
            ::std::result::Result::Ok(x) => x,
            ::std::result::Result::Err(e) => {
                $crate::dart_handle::Error::propagate_error(e);
                #[allow(unused_unsafe)]
                unsafe {
                    ::std::hint::unreachable_unchecked()
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_from {
    ($new_ty:ty, ($this:ty), $($t:ty),*) => {
        $(
            impl From<$t> for $this {
                fn from(value: $t) -> Self {
                    Self::new(value as $new_ty)
                }
            }
        )*
    }
}
