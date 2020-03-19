use crate::dart_handle::{DartHandle, UnverifiedDartHandle};
use dart_sys as ffi;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, UnwindSafe};
use std::sync::RwLock;

pub mod dart_cobject;
pub mod dart_handle;
pub mod dart_native_arguments;
pub mod dart_types;
pub mod prelude;

extern crate mashup;

lazy_static! {
    ///
    /// The global register for functions.
    ///
    /// This is searched whenever a function is asked for.
    ///
    static ref REGISTER: RwLock<FunctionRegister> = RwLock::new(FunctionRegister::default());
}

pub type NativeFunction = unsafe extern "C" fn(arguments: ffi::Dart_NativeArguments);

///
/// Stores a two-way connection between functions and their names.
///
/// This is created and loaded at startup in the `init` function.
///
#[derive(Default)]
#[doc(hidden)]
pub struct FunctionRegister {
    ///
    /// The name -> function connection.
    ///
    functions: HashMap<&'static CStr, NativeFunction>,
    ///
    /// The function -> name connection.
    ///
    function_names: HashMap<NativeFunction, &'static CStr>,
}

impl FunctionRegister {
    ///
    /// Adds a function into the register. Leaks the name and puts
    /// it into both `HashMap`s.
    ///
    pub fn add_function(&mut self, function: NativeFunction, name: &str) {
        //Convert name to cstring
        let name = CString::new(name).unwrap();
        //SAFETY:
        // We leak the bytes, therefore making them live forever.
        // Since the `CString` has allocated the name on the heap
        // we can just leak the bytes and they will live forever.
        // Unsafe is required since we need to build a `CStr` from
        // the bytes, but this is safe since they just came out of
        // a `CString`, and also includes the `nul` byte.
        let name = unsafe {
            let boxed_slice = name.into_bytes_with_nul().into_boxed_slice();
            let leaked = Box::leak::<'static>(boxed_slice);
            CStr::from_bytes_with_nul_unchecked(&*leaked)
        };
        self.functions.insert(name, function);
        self.function_names.insert(function, name);
    }

    ///
    /// Gets a function given a name.
    ///
    /// # SAFETY:
    ///  `name` must be a valid pointer to a nul-terminated C-string.
    ///
    pub unsafe fn get_function(&self, name: *const c_char) -> ffi::Dart_NativeFunction {
        let name = CStr::from_ptr::<'static>(name);
        self.functions.get(name).cloned()
    }

    ///
    /// Gets a name given a function.
    ///
    pub(crate) fn get_name_from_function(
        &self,
        function: ffi::Dart_NativeFunction,
    ) -> Option<&'static CStr> {
        function
            .and_then(|x| self.function_names.get(&x))
            .map(|x| *x)
    }
}

///
/// Registers a set of functions given mutable access to the global
/// `FunctionRegister`. Generate this using `export_dart_functions`.
///
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct Registerer {
    ///
    /// Macro-generated function which adds all the functions into
    /// the function register.
    ///
    pub export_fn: fn(&mut FunctionRegister),
}

///
/// Adds all the functions into the global register. This effectively
/// just calls all of the internal functions within the `Registerer`s
/// passed to it, but it also takes care of some dart-vm setup stuff
/// for native extensions.
///
/// # Parameters
/// - **`parent_library`** is the handle to the library which has loaded
///   this module. Should it be an error, it will be immediately returned.
///
/// - **`registers`** are the functions register-ers which will add to the
///   global register. These are created using `export_dart_functions`,
///   and are passed into the `create_init_function`.
///
/// # Safety
///
/// `parent_library` must be a valid `Dart_Handle`. Not doing so will cause
/// the VM to invoke UB.
///
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

    //Sets the appropriate resolvers for the library.
    let result_code = ffi::Dart_SetNativeResolver(
        parent_library.handle(),  //Library
        Some(resolve_name),      //Name -> fn
        Some(resolve_function),  //fn -> Name
    );

    UnverifiedDartHandle::new(result_code)
        .get_error()
        .map(|_| UnverifiedDartHandle::null()) // Ok is null.
        .handle()
}

///
/// Searches the global register for a function.
///
/// # Parameters
///
/// - **`name`** is a Dart String with the name of the function.
///
/// - **`_argc`** is the number of parameters in the function. Currently unused.
///
/// - **`auto_scope_setup`** is a flag which signals whether the VM should setup
///   a scope for this function. This will be set to true by default.
///
/// # Safety
///
/// `name` Must be a valid dart handle. Not doing so will incur the VM to invoke UB.
///
/// Also, `auto_scope_setup` must be a valid pointer. The data underneath it
/// does not need to be a valid `bool` since its destructor is not run and
/// LLVM cannot claim UB.
///
#[allow(dead_code)] //Usage of this function is declared in external crates.
// TODO: Implement argument counting
unsafe extern "C" fn resolve_name(
    name: ffi::Dart_Handle,
    _argc: std::os::raw::c_int,
    auto_scope_setup: *mut bool,
) -> ffi::Dart_NativeFunction {
    let name = UnverifiedDartHandle::new(name).get_error().ok()?;

    if !name.is_string() {
        return None;
    }

    std::ptr::write(auto_scope_setup, true);

    // Go directly through the UnverifiedDartHandle interface since
    // we actually need a `CString`, and `DString` only gives us
    // regular `String`s.
    let cname = dart_unwrap!(name.to_string());

    REGISTER.read().unwrap().get_function(cname.as_ptr())
}

///
/// Finds a function's name given its pointer in the global register.
///
extern "C" fn resolve_function(function: ffi::Dart_NativeFunction) -> *const u8 {
    let name = REGISTER.read().unwrap().get_name_from_function(function);
    if let Some(x) = name {
        x.as_ptr() as *const u8
    } else {
        b"<Unknown Native Function>\0" as &[u8] as *const [u8] as *const u8
    }
}

///
/// Runs a synchronous function and protects against unwinding
/// into C stack frames.
///
/// # Parameters
///
/// - **`f`** is the function that the user is wrapping. It takes
///   a `NativeArguments`. This function will eventually be able
///   to serialize the `NativeArguments` into the types that the
///   function requires. For now you're stuck doing that yourself.
///
/// - **`value`** is the parameters with which `f` is intended to be
///   called. This function will wrap this and send it to `f`.
///
/// # Safety
///
/// This function requires `value` to be a valid pointer to function
/// arguments.
///
#[doc(hidden)]
pub unsafe fn catch_panic_hook(
    f: impl FnOnce(crate::dart_native_arguments::NativeArguments) + UnwindSafe,
    value: ffi::Dart_NativeArguments,
) {
    let result = catch_unwind(move || {
        f(crate::dart_native_arguments::NativeArguments::new(value))
    });
    if let Err(e) = result {
        let msg;
        match e.downcast_ref::<String>() {
            Some(x) => msg = &**x,
            None => match e.downcast::<&str>() {
                Ok(x) => msg = *x,
                Err(_e) => msg = "Panic of unknown nature in Rust code!",
            },
        }

        let error = crate::dart_handle::Error::new_api(msg).unwrap();
        error.propagate_error();
    }
}

///
/// Creates and returns a `SendPort` for an asynchronous function.
///
/// # Parameters
///
/// - **`f`** is the asynchronous function which is being wrapped.
///   It is placed into the callback for the `SendPort` and will get
///   messages when `SendPort.send(dynamic)` is called.
///
///   ## **`f`** Parameters
///
///   - **`dest_port_id`** is the port which the messages will have
///     been sent to.
///
///   - **`message`** is the message which has been sent to the function.
///     This is effectively its parameters.
///
/// - **`value`** is the arguments with which the function has been
///   called. This is used to set a return with the appropriate
///   `SendPort`.
///
/// - **`name`** is used as the name of the service.
///
/// # Safety
///
/// This function requires the same safety as `catch_panic_hook`, since
/// it just calls it. It must also be safe to call `f` in the asynchronous
/// context it will be called in.
///
#[doc(hidden)]
pub unsafe fn catch_panic_hook_async(
    f: unsafe extern "C" fn(
        dest_port_id: ffi::Dart_Port,
        message: *mut ffi::Dart_CObject,
    ),
    value: ffi::Dart_NativeArguments,
    name: &str,
) {
    catch_panic_hook(
        |x| {
            crate::dart_handle::enter_scope();
            let name = CString::new(name).unwrap_or_else(|e| {
                crate::dart_handle::exit_scope();
                panic!("Name is invalid: `{}`", e);
            });
            let service_port =
                crate::dart_handle::NativePort::new_native(name.clone(), f)
                    .unwrap_or_else(|| {
                        crate::dart_handle::exit_scope();
                        panic!("Name is invalid: `{:?}`", name);
                    });
            let (_, send_port_instance) =
                crate::dart_handle::Port::new(service_port.port()).unwrap();
            x.set_return(send_port_instance);
            crate::dart_handle::exit_scope();
        },
        value,
    );
}

///
/// Catches a panic from a function from unwinding across C frames.
///
/// This serves the same purpose as `catch_panic_hook`, but is
/// for `async` purposes since it seems that these function calls
/// are a bit different. All that I could find with respect to this
/// is that we should abort the process instead of returning an
/// error.
///
#[doc(hidden)]
pub unsafe fn catch_async_panic(
    func: fn(crate::dart_cobject::CObject, crate::dart_handle::Port),
    port: ffi::Dart_Port,
    message: *mut ffi::Dart_CObject,
) {
    let result = catch_unwind(move ||
        func(
            crate::dart_cobject::CObject::from(*message),
            crate::dart_handle::Port::from_port(port).unwrap(),
        )
    );
    // We can ignore the error message since it will already have been printed.
    if result.is_err() {
        eprintln!("Rust panicked in an unwind-unsafe way. Aborting the process.");
        std::process::abort();
    }
}

// TODO: Namespacing using `concat!` and `stringify!`
///
/// Creates a `Registerer` which exports Dart native extension functions.
///
/// # Usage
/// - Create your functions.
///   ```
///   use dart::prelude::*;
///   use std::ffi::CString;
///   fn my_function(args: NativeArguments) {
///       args.set_return(DString::new("Hello, World").safe_handle());
///   }
///   fn my_async_function(message: CObject, _from_port: Port) {
///       if let CObject::SendPort(port) = message {
///           let port = unsafe { Port::from_port(port.0.id) }.unwrap();
///           port.post_cobject(CObject::String(CString::new("Hello, Async World").unwrap()));
///       } else {
///           panic!("Didn't get a port to reply!");
///       }
///   }
///   ```
/// - Export your functions using the syntax
///   ```
///   # use dart::prelude::*;
///   # use std::ffi::CString;
///   # fn my_function(args: NativeArguments) {
///   #     args.set_return(DString::new("Hello, World").safe_handle());
///   # }
///   # fn my_async_function(message: CObject, _from_port: Port) {
///   #     if let CObject::SendPort(port) = message {
///   #         let port = unsafe { Port::from_port(port.0.id) }.unwrap();
///   #         port.post_cobject(CObject::String(CString::new("Hello, Async World").unwrap()));
///   #     } else {
///   #         panic!("Didn't get a port to reply!");
///   #     }
///   # }
///   // Creates a set of exports called `my_exports`.
///   dart::export_dart_functions!(my_exports:
///       ["function1" -> my_function],
///       ["function2service_port" -> my_async_function as async]
///   );
///   ```
///
#[macro_export]
macro_rules! export_dart_functions {
    ($export_name:ident: $([$name:literal -> $function:ident $(as $a_sync:tt)?]),*$(,)?) => {
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
                            // TODO: Implement some way to automatically convert arguments.
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
            $crate::catch_async_panic($func, dest_port_id, message);
        }
        $crate::catch_panic_hook_async($async_name, $args, $registered_name)
    };
}

///
/// Creates an appropriate init function given the registerers
/// and final library name.
///
/// # Usage
/// - Create a set of exported functions using [`export_dart_functions`].
/// - Pass the export name assigned to in `export_dart_functions` into this
///   macro, along with the final library name for the correct initialization
///   function name. This will have to match the resulting binary's file name
///   and will be searched for by the Dart VM.
///   ```
///   # dart::export_dart_functions!(my_exports: );
///   dart::create_init_function!(library_name, [my_exports]);
///   ```
///
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

///
/// Unwraps a result, propagating the Dart error, should it be
/// present. This will never return if it happens to encounter
/// an `Err(e)` variant.
///
/// # Usage
/// ```no_run
/// # use dart::prelude::*;
/// let my_list = List::new_data(vec![1u8, 2, 3, 4]);
/// let result = my_list.get_at(4);
/// let int = dart_unwrap!(result); // Oh noes! We got an error!
/// println!("{:?}", *int); // This will never run.
/// ```
///
#[macro_export]
macro_rules! dart_unwrap {
    ($x: expr) => {
        match {
            let y: Result<_, $crate::dart_handle::Error> = $x;
            y
        } {
            ::std::result::Result::Ok(x) => x,
            ::std::result::Result::Err(e) => {
                $crate::dart_handle::Error::propagate_error(e);
                #[allow(unused_unsafe)]
                unsafe {
                    ::std::hint::unreachable_unchecked()
                }
            }
        }
    };
}
