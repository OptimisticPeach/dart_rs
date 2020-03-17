use std::collections::HashMap;

use std::sync::RwLock;

use dart_sys as ffi;

use lazy_static::lazy_static;
use std::ffi::{CStr, CString};
use crate::dart_handle::{UnverifiedDartHandle, DartHandle};
use std::os::raw::c_char;
use std::panic::{catch_unwind, UnwindSafe};

pub mod dart_handle;
pub mod dart_native_arguments;
pub mod dart_cobject;
pub mod dart_types;
pub mod prelude;

extern crate mashup;

lazy_static! {
    static ref REGISTER: RwLock<FunctionRegister> = RwLock::new(FunctionRegister::default());
}

pub type NativeFunction = unsafe extern "C" fn(arguments: ffi::Dart_NativeArguments);

#[derive(Default)]
#[doc(hidden)]
pub struct FunctionRegister {
    functions: HashMap<&'static CStr, NativeFunction>,
    function_names: HashMap<NativeFunction, &'static CStr>,
}

impl FunctionRegister {
    pub fn add_function(&mut self, function: NativeFunction, name: &str) {
        let name = CString::new(name).unwrap();
        let name = unsafe {
            CStr::from_bytes_with_nul_unchecked(&*Box::leak::<'static>(name.into_bytes_with_nul().into_boxed_slice()))
        };
        self.functions.insert(name, function);
        self.function_names.insert(function, name);
    }

    pub unsafe fn get_function(&self, name: *const c_char) -> ffi::Dart_NativeFunction {
        let name = CStr::from_ptr::<'static>(name);
        self.functions.get(name).cloned()
    }

    pub(crate) fn get_name_from_function(&self, function: ffi::Dart_NativeFunction) -> Option<&'static CStr> {
        function.and_then(|x| self.function_names.get(&x)).map(|x| *x)
    }
}

#[allow(dead_code)] //Usage of this field is declared in external crates.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct Registerer {
    pub export_fn: fn(&mut FunctionRegister),
}

#[allow(dead_code)] //Usage of this function is declared in external crates.
#[doc(hidden)]
pub unsafe fn init(parent_library: ffi::Dart_Handle, registers: &[Registerer]) -> ffi::Dart_Handle {
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

    println!("{:?}", cname);

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

#[doc(hidden)]
pub fn catch_panic_hook(f: impl FnOnce(crate::dart_native_arguments::NativeArguments) + UnwindSafe, value: ffi::Dart_NativeArguments) {
    let result = catch_unwind(
        move || f(unsafe { crate::dart_native_arguments::NativeArguments::new(value) })
    );
    if let Err(e) = result {
        let msg;
        match e.downcast_ref::<String>() {
            Some(x) => msg = &**x,
            None => {
                match e.downcast::<&str>() {
                    Ok(x) => msg = *x,
                    Err(_e) => msg = "Panic of unknown nature in Rust code!",
                }
            }
        }

        let error = crate::dart_handle::Error::new_api(msg).unwrap();
        error.propagate_error();
    }
}

#[doc(hidden)]
pub fn catch_panic_hook_async(f: unsafe extern "C" fn(dest_port_id: ::dart_sys::Dart_Port, message: *mut ::dart_sys::Dart_CObject), value: ffi::Dart_NativeArguments, name: &str) {
    catch_panic_hook(|x| {
        unsafe {crate::dart_handle::enter_scope()};
        let name = CString::new(name).unwrap_or_else(
            |e| {
                unsafe {crate::dart_handle::exit_scope()};
                panic!("Name is invalid: `{}`", e);
            }
        );
        let service_port = unsafe {crate::dart_handle::NativePort::new_native(name.clone(), f)}.unwrap_or_else(
            || {
                unsafe { crate::dart_handle::exit_scope() };
                panic!("Name is invalid: `{:?}`", name);
            }
        );
        let (_, send_port_instance) = unsafe {crate::dart_handle::Port::new(service_port.port())}.unwrap();
        x.set_return(send_port_instance);
        unsafe { crate::dart_handle::exit_scope() };
    }, value);
}

#[doc(hidden)]
pub fn catch_async_panic(func: fn(crate::dart_cobject::CObject, crate::dart_handle::Port), port: ffi::Dart_Port, message: *mut ffi::Dart_CObject) {
    let result = catch_unwind(
        move || {
            unsafe {
                func(
                    crate::dart_cobject::CObject::from(*message),
                    crate::dart_handle::Port::from_port(port).unwrap()
                )
            }
        }
    );
    // We can ignore the error message since it will already have been printed.
    if result.is_err() {
        eprintln!("Rust panicked in an unwind-unsafe way. Aborting the process.");
        std::process::abort();
    }
}

#[macro_export]
macro_rules! export_dart_functions {
    ($export_name:ident: $([$name:literal -> $function:ident $(as $a_sync:tt)?]),*) => {
        use mashup::*;
        #[allow(non_snake_case, non_upper_case_globals)]
        static $export_name: $crate::Registerer = $crate::Registerer {
            export_fn: {
                mashup! {
                    $(
                        #[macro_export]
                        $function["n"] = $function _name;
                        #[macro_export]
                        $function["n_async"] = async_ $function _name;
                    )*
                }
                fn register_all(register: &mut $crate::FunctionRegister) {
                    $(
                        $function! {
                            unsafe extern "C" fn "n"(x: ::dart_sys::Dart_NativeArguments) {
                                export_dart_functions!(@$($a_sync as ("n_async", $name))?, $function, x);
                            }
                            register.add_function("n", $name);
                        }
                    )*
                }
                register_all
            }
        };
    };
    (@, $func:ident, $args:ident) => {
        $crate::catch_panic_hook($func, $args);
    };
    (@async as ($async_name:ident, $registered_name:literal), $func:ident, $args:ident) => {
        unsafe extern "C" fn $async_name(dest_port_id: ::dart_sys::Dart_Port, message: *mut ::dart_sys::Dart_CObject) {
            let _: fn(args: $crate::dart_cobject::CObject, reply: $crate::dart_handle::Port) = $func;
            $func($crate::dart_cobject::CObject::from(*message), $crate::dart_handle::Port::from_port(dest_port_id).expect("Invalid dest_port_id!"));
        }
        $crate::catch_panic_hook_async($async_name, $args, $registered_name)
    };
}

#[macro_export]
macro_rules! create_init_function {
    ($crate_name:ident, [$($name:ident),*$(,)?]) => {
        use mashup::*;
        ::mashup::mashup! {
            dart_rs_init_name["init"] = $crate_name _Init;
        }
        dart_rs_init_name! {
            #[allow(non_snake_case, unused_variables)]
            #[no_mangle]
            unsafe extern "C" fn "init"(parent_library: ::dart_sys::Dart_Handle) -> ::dart_sys::Dart_Handle {
                $crate::init(parent_library, &[$($name),*])
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
