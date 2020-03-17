use dart_sys as ffi;
use crate::dart_handle::{Port, TypedData};
use std::ffi::{CString, CStr};
use std::any::Any;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ops::{Index, IndexMut};

pub enum CObject {
    Null,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Double(f64),
    String(CString),
    SendPort(Sender),
    Array(Vec<Self>),
    TypedData(TypedDataArray<dyn Any>)
}

impl CObject {
    pub unsafe fn from(ffi::Dart_CObject {type_: ty, value}: ffi::Dart_CObject) -> Self {
        use ffi::Dart_CObject_Type::*;
        match ty {
            Null => CObject::Null,
            Bool => CObject::Bool(value.as_bool),
            Int32 => CObject::Int32(value.as_int32),
            Int64 => CObject::Int64(value.as_int64),
            Double => CObject::Double(value.as_double),
            SendPort => CObject::SendPort(Sender(value.as_send_port)),
            String => {
                let ptr = value.as_string;
                let cstr = CStr::from_ptr(ptr);
                CObject::String(cstr.to_owned())
            },
            Array => {
                let arr = value.as_array;
                let ptr = arr.values;
                let len = arr.length as usize;
                let slice = std::slice::from_raw_parts_mut(ptr, len);
                let vec = slice
                    .iter()
                    .map(|x| Self::from(**x))
                    .collect::<Vec<_>>();
                CObject::Array(vec)
            },
            TypedData => CObject::TypedData(TypedDataArray::new(value.as_typed_data)),
            ExternalTypedData => CObject::TypedData(TypedDataArray::new_external(value.as_external_typed_data)),
            Unsupported => panic!("Unsupported CObject!"),
            NumberOfTypes => unimplemented!("Number of Typed has yet to be implemented!"),
            Capability => unimplemented!("Capabilities within CObjects have yet to be implemented!"),
        }
    }

    pub fn into_leak(self) -> ffi::Dart_CObject {
        use dart_sys::Dart_CObjectValue;
        match self {
            CObject::Null => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Null, value: Dart_CObjectValue { as_bool: false } },
            CObject::Bool(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Bool, value: Dart_CObjectValue { as_bool: x } },
            CObject::Int32(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Int32, value: Dart_CObjectValue { as_int32: x } },
            CObject::Int64(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Int64, value: Dart_CObjectValue { as_int64: x } },
            CObject::Double(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Double, value: Dart_CObjectValue { as_double: x } },
            CObject::TypedData(x) => {
                match x {
                    TypedDataArray::WithoutFinalizer(x, _) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::TypedData, value: Dart_CObjectValue { as_typed_data: x}},
                    TypedDataArray::WithFinalizer(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::ExternalTypedData, value: Dart_CObjectValue { as_external_typed_data: x } },
                }
            },
            CObject::SendPort(Sender(x)) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::SendPort, value: Dart_CObjectValue { as_send_port: x } },
            CObject::Array(x) => {
                let vec: Vec<Box<ffi::Dart_CObject>> = x
                    .into_iter()
                    .map(|x| x.into_leak())
                    .map(Box::new)
                    .collect();
                let boxed = Box::leak(vec.into_boxed_slice());
                let ptr =
                    boxed as *mut [_]
                          as *mut [Box<ffi::Dart_CObject>]
                          as *mut [*mut ffi::Dart_CObject]
                          as *mut *mut ffi::Dart_CObject;
                let len = boxed.len();
                let array = ffi::Dart_Array {
                    length: len as _,
                    values: ptr,
                };
                ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Array, value: Dart_CObjectValue { as_array: array } }
            },
            CObject::String(cstring) => {
                let ptr = CString::into_raw(cstring);
                ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::String, value: Dart_CObjectValue { as_string: ptr } }
            },
        }
    }

    pub fn as_non_leak(&'_ self) -> CObjectLock<'_> {
        use dart_sys::Dart_CObjectValue;
        let obj = match self {
            CObject::Null => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Null, value: Dart_CObjectValue { as_bool: false } },
            CObject::Bool(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Bool, value: Dart_CObjectValue { as_bool: *x } },
            CObject::Int32(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Int32, value: Dart_CObjectValue { as_int32: *x } },
            CObject::Int64(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Int64, value: Dart_CObjectValue { as_int64: *x } },
            CObject::Double(x) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Double, value: Dart_CObjectValue { as_double: *x } },
            CObject::TypedData(x) => {
                match x {
                    TypedDataArray::WithoutFinalizer(x, _) => ffi::Dart_CObject {
                        type_: ffi::Dart_CObject_Type::TypedData,
                        value: Dart_CObjectValue { as_typed_data: *x }
                    },
                    TypedDataArray::WithFinalizer(x) => ffi::Dart_CObject {
                        type_: ffi::Dart_CObject_Type::ExternalTypedData,
                        value: Dart_CObjectValue { as_external_typed_data: *x }
                    },
                }
            },
            CObject::SendPort(Sender(x)) => ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::SendPort, value: Dart_CObjectValue { as_send_port: *x } },
            CObject::Array(x) => {
                let vec: Vec<Box<ffi::Dart_CObject>> = x
                    .into_iter()
                    .map(|x| x.as_non_leak().object)
                    .map(Box::new)
                    .collect();
                let boxed = Box::leak(vec.into_boxed_slice());
                let ptr =
                    boxed as *mut [_]
                        as *mut [Box<ffi::Dart_CObject>]
                        as *mut [*mut ffi::Dart_CObject]
                        as *mut *mut ffi::Dart_CObject;
                let len = boxed.len();
                let array = ffi::Dart_Array {
                    length: len as _,
                    values: ptr,
                };
                ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::Array, value: Dart_CObjectValue { as_array: array } }
            },
            CObject::String(cstring) => {
                let ptr = cstring.as_ptr() as *mut i8;
                ffi::Dart_CObject { type_: ffi::Dart_CObject_Type::String, value: Dart_CObjectValue { as_string: ptr } }
            },
        };

        unsafe {
            CObjectLock::new(self, obj)
        }
    }
}

pub struct CObjectLock<'a> {
    _rust_cobject: &'a CObject,
    pub(crate) object: ffi::Dart_CObject,
}

impl<'a> CObjectLock<'a> {
    pub unsafe fn new(rust_cobject: &'a CObject, object: ffi::Dart_CObject) -> Self {
        Self {
            _rust_cobject: rust_cobject,
            object
        }
    }
    pub fn post_onto(&mut self, sender: &mut Sender) -> bool {
        unsafe {
            let port = Port::from_port(sender.0.id);
            if let Some(port) = port {
                port.post_cobject(&mut self.object)
            } else {
                false
            }
        }
    }
}

#[repr(transparent)]
pub struct Sender(pub ffi::Dart_SendPort);

#[derive(Copy, Clone)]
pub enum TypedDataArray<T: ?Sized> {
    WithoutFinalizer(ffi::Dart_TypedData, PhantomData<T>),
    WithFinalizer(ffi::Dart_ExternalTypedData),
}

impl TypedDataArray<dyn Any> {
    pub unsafe fn new(arr: ffi::Dart_TypedData) -> Self {
        TypedDataArray::WithoutFinalizer(arr, PhantomData)
    }

    pub unsafe fn new_external(arr: ffi::Dart_ExternalTypedData) -> Self {
        TypedDataArray::WithFinalizer(arr)
    }

    pub fn cast<T: TypedData>(self) -> Option<TypedDataArray<T>> {
        match self {
            TypedDataArray::WithFinalizer(x) => {
                if x.type_ == T::TYPE {
                    Some(TypedDataArray::WithFinalizer(x))
                } else {
                    None
                }
            },
            TypedDataArray::WithoutFinalizer(x, _) => {
                if x.type_ == T::TYPE {
                    Some(TypedDataArray::WithoutFinalizer(x, PhantomData))
                } else {
                    None
                }
            },
        }
    }
}

impl<T: TypedData> TypedDataArray<T> {
    pub fn create(data: Vec<T>) -> Self {
        let ptr = Box::leak(data.into_boxed_slice());
        let len = ptr.len();
        let ptr_ptr = Box::leak(Box::new(ptr as *mut [T]));

        unsafe extern "C" fn free<T>(_isolate_callback_data: *mut c_void, _handle: ffi::Dart_WeakPersistentHandle, peer: *mut c_void) {
            let ptr = peer as *mut *mut [T];
            let boxed = Box::from_raw(*ptr);
            drop(boxed);
            let boxed_2 = Box::from_raw(ptr);
            drop(boxed_2);
        }

        TypedDataArray::WithFinalizer(
            ffi::Dart_ExternalTypedData {
                type_: T::TYPE,
                length: len as _,
                data: ptr as *mut [T] as *mut T as *mut u8,
                peer: ptr_ptr as *mut *mut [T] as *mut c_void,
                callback: Some(free::<T>)
            }
        )
    }

    pub fn recast(self) -> TypedDataArray<dyn Any> {
        match self {
            TypedDataArray::WithFinalizer(x) => unsafe {TypedDataArray::new_external(x)},
            TypedDataArray::WithoutFinalizer(x, _) => unsafe {TypedDataArray::new(x)}
        }
    }
}

impl<T: TypedData + Sized> Index<usize> for TypedDataArray<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        use TypedDataArray::*;
        match self {
            WithoutFinalizer(ffi::Dart_TypedData {length, values, ..}, _) |
            WithFinalizer(ffi::Dart_ExternalTypedData {length, data: values, ..}) => {
                unsafe {
                    let slice = std::slice::from_raw_parts(*values as *mut T, *length as _);
                    &slice[idx]
                }
            }
        }
    }
}

impl<T: TypedData + Sized> IndexMut<usize> for TypedDataArray<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        use TypedDataArray::*;
        match self {
            WithoutFinalizer(ffi::Dart_TypedData { length, values, .. }, _) |
            WithFinalizer(ffi::Dart_ExternalTypedData { length, data: values, .. }) => {
                unsafe {
                    let slice = std::slice::from_raw_parts_mut(*values as *mut T, *length as _);
                    &mut slice[idx]
                }
            }
        }
    }
}
