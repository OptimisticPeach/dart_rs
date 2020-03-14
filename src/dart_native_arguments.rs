use dart_sys as ffi;
use crate::dart_handle::{UnverifiedDartHandle, DartHandle};
use crate::dart_handle::Error;
use std::mem::MaybeUninit;
use std::ffi::CStr;
use crate::dart_types::d_string::DString;

#[repr(transparent)]
pub struct NativeArguments {
    args: ffi::Dart_NativeArguments
}

impl NativeArguments {
    pub unsafe fn new(args: ffi::Dart_NativeArguments) -> Self {
        Self { args }
    }

    pub fn get_native_arguments(&self) -> Result<(Vec<ffi::Dart_NativeArgument_Descriptor>, Vec<ffi::Dart_NativeArgument_Value>), Error> {
        let len = self.get_native_argument_count();
        let mut types = Vec::with_capacity(len);
        let mut values = Vec::with_capacity(len);
        unsafe {
            let handle = ffi::Dart_GetNativeArguments(
                self.args,
                len as _,
                types.as_mut_ptr(),
                values.as_mut_ptr(),
            );
            let error_handle = UnverifiedDartHandle::new(handle).get_error();
            match error_handle {
                Ok(_) => {
                    types.set_len(len);
                    values.set_len(len);
                    Ok((types, values))
                },
                Err(e) => {
                    std::mem::forget(types);
                    std::mem::forget(values);
                    Err(e)
                }
            }
        }
    }

    pub fn get_native_argument_count(&self) -> usize {
        unsafe {
            ffi::Dart_GetNativeArgumentCount(self.args) as _
        }
    }

    pub fn get_native_argument(&self, idx: usize) -> UnverifiedDartHandle {
        unsafe {
            UnverifiedDartHandle::new(ffi::Dart_GetNativeArgument(self.args, idx as _))
        }
    }

    pub fn get_string_arg(&self, idx: usize) -> Result<String, Error> {
        unsafe {
            let mut peer = MaybeUninit::uninit();
            let handle = ffi::Dart_GetNativeStringArgument(self.args, idx as _, peer.as_mut_ptr());
            let handle = UnverifiedDartHandle::new(handle).get_error();
            match handle {
                Ok(x) => {
                    if x.is_string() {
                        Ok(x.string_to_utf8()?)
                    } else {
                        let cstr = CStr::from_ptr(peer.assume_init() as *mut i8);
                        let cstring = cstr.to_owned();
                        let string = cstring.into_string().unwrap();
                        Ok(string)
                    }
                },
                Err(e) => Err(e)
            }
        }
    }

    pub fn get_bool_arg(&self, idx: usize) -> Result<bool, Error> {
        unsafe {
            let mut val = MaybeUninit::uninit();
            let handle = ffi::Dart_GetNativeBooleanArgument(self.args, idx as _, val.as_mut_ptr());
            UnverifiedDartHandle::new(handle).get_error().map(|_| val.assume_init())
        }
    }

    pub fn get_i64_arg(&self, idx: usize) -> Result<i64, Error> {
        unsafe {
            let mut val = MaybeUninit::uninit();
            let handle = ffi::Dart_GetNativeIntegerArgument(self.args, idx as _, val.as_mut_ptr());
            UnverifiedDartHandle::new(handle).get_error().map(|_| val.assume_init())
        }
    }

    pub fn get_f64_arg(&self, idx: usize) -> Result<f64, Error> {
        unsafe {
            let mut val = MaybeUninit::uninit();
            let handle = ffi::Dart_GetNativeDoubleArgument(self.args, idx as _, val.as_mut_ptr());
            UnverifiedDartHandle::new(handle).get_error().map(|_| val.assume_init())
        }
    }

    pub fn set_return(&self, val: UnverifiedDartHandle) {
        unsafe {
            ffi::Dart_SetReturnValue(self.args, val.handle())
        }
    }

    pub fn set_bool_return(&self, val: bool) {
        unsafe {
            ffi::Dart_SetBooleanReturnValue(self.args, val);
        }
    }

    pub fn set_i64_return(&self, val: i64) {
        unsafe {
            ffi::Dart_SetIntegerReturnValue(self.args, val);
        }
    }

    pub fn set_f64_return(&self, val: f64) {
        unsafe {
            ffi::Dart_SetDoubleReturnValue(self.args, val);
        }
    }
}

pub struct NativeArgumentDescriptor {
    pub ty: ffi::Dart_NativeArgument_Type,
    pub idx: u8
}

#[derive(Clone)]
#[non_exhaustive]
pub enum NativeArgumentValue {
    #[doc(hidden)]
    Null,
    Bool(bool),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    String(DString),
    Instance(UnverifiedDartHandle),
}

impl NativeArgumentValue {
    pub fn get_args(args: NativeArguments) -> Result<Vec<Self>, Error> {
        let (descriptors, values) = args.get_native_arguments()?;
        assert_eq!(descriptors.len(), values.len());
        let mut result = vec![NativeArgumentValue::Null; descriptors.len()];
        for (desc, val) in descriptors.into_iter().zip(values.into_iter()) {
            use ffi::Dart_NativeArgument_Type::*;
            let idx = desc.index;
            let next = unsafe {
                match desc.type_ {
                    Bool => NativeArgumentValue::Bool(val.as_bool),
                    Int32 => NativeArgumentValue::Int32(val.as_int32),
                    Uint32 => NativeArgumentValue::UInt32(val.as_uint32),
                    Int64 => NativeArgumentValue::Int64(val.as_int64),
                    Uint64 => NativeArgumentValue::UInt64(val.as_uint64),
                    Double => NativeArgumentValue::Double(val.as_double),
                    String => {
                        let string = val.as_string;
                        let d_string = DString::from_handle(UnverifiedDartHandle::new(string.dart_str).get_error()?);
                        let d_string = d_string.ok().unwrap();
                        NativeArgumentValue::String(d_string)
                    },
                    Instance => NativeArgumentValue::Instance(UnverifiedDartHandle::new(val.as_instance).get_error()?),
                    NativeFields => panic!("Native fields are not supported."),
                }
            };
            result[idx as usize] = next;
        }
        for arg in result.iter() {
            if let NativeArgumentValue::Null = arg {
                panic!("Unfilled argument in call to native function!");
            }
        }
        Ok(result)
    }
}
