use dart_sys as ffi;
use std::ops::Deref;
use std::ffi::{CStr, CString, NulError};
use std::convert::{Infallible, TryInto};
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_void};
use dart_sys::Dart_CObject;
use std::fmt::{Debug, Formatter};

pub unsafe trait DartHandle: 'static + Sized {
    fn handle(&self) -> ffi::Dart_Handle;
    fn safe_handle(&self) -> UnverifiedDartHandle;
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle>;
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct UnverifiedDartHandle {
    handle: ffi::Dart_Handle,
}

unsafe impl DartHandle for UnverifiedDartHandle {
    fn handle(&self) -> ffi::Dart_Handle {
        self.handle
    }
    fn safe_handle(&self) -> Self {*self}
    fn from_handle(handle: Self) -> Result<Self, Self> { Ok(handle) }
}

impl UnverifiedDartHandle {
    pub unsafe fn new(handle: ffi::Dart_Handle) -> Self {
        assert_ne!(handle, std::ptr::null_mut());
        Self {
            handle
        }
    }

    pub fn get_error(self) -> Result<Self, Error> {
        unsafe {
            if ffi::Dart_IsError(self.handle) {
                if ffi::Dart_IsUnhandledExceptionError(self.handle) {
                    Err(Error::of(self, ErrorKind::UnhandledException))
                } else if ffi::Dart_IsApiError(self.handle) {
                    Err(Error::of(self, ErrorKind::Api))
                } else if ffi::Dart_IsCompilationError(self.handle) {
                    Err(Error::of(self, ErrorKind::Compilation))
                } else if ffi::Dart_IsFatalError(self.handle) {
                    Err(Error::of(self, ErrorKind::Fatal))
                } else {
                    panic!("This shouldn't ever happen!");
                }
            } else {
                Ok(self)
            }
        }
    }

    pub fn to_string(&self) -> Result<CString, Error> {
        unsafe {
            let string_handle = ffi::Dart_ToString(self.handle);
            let string_handle = Self::new(string_handle).get_error()?;
            let mut cstr = MaybeUninit::<*const c_char>::uninit();
            let string_error =
                ffi::Dart_StringToCString(string_handle.handle, cstr.as_mut_ptr());
            Self::new(string_error).get_error()?;
            let cstr = cstr.assume_init();
            let cstring = CStr::from_ptr(cstr).to_owned();
            Ok(cstring)
        }
    }

    pub fn identity_eq(a: Self, b: Self) -> bool {
        unsafe {
            ffi::Dart_IdentityEquals(a.handle, b.handle)
        }
    }

    pub fn handle_message() -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_HandleMessage();
            Self::new(handle).get_error()
        }
    }

    pub fn wait_for_event(timeout_millis: i64) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_WaitForEvent(timeout_millis);
            Self::new(handle).get_error()
        }
    }

    pub fn null() -> Self {
        unsafe {
            Self::new(ffi::Dart_Null())
        }
    }
    pub fn is_null(&self) -> bool {
        unsafe {
            ffi::Dart_IsNull(self.handle)
        }
    }

    pub fn empty_string() -> Self {
        unsafe {
            Self::new(ffi::Dart_EmptyString())
        }
    }

    pub fn equals(&self, other: Self) -> Result<bool, Error> {
        unsafe {
            let mut result = MaybeUninit::uninit();
            let error_handle = ffi::Dart_ObjectEquals(self.handle, other.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn instanceof(&self, ty: Self) -> Result<bool, Error> {
        unsafe {
            let mut result = MaybeUninit::uninit();
            let error_handle = ffi::Dart_ObjectIsType(self.handle, ty.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn is_instance(&self) -> bool {
        unsafe {
            ffi::Dart_IsInstance(self.handle)
        }
    }

    pub fn is_integer(&self) -> bool {
        unsafe {
            ffi::Dart_IsInteger(self.handle)
        }
    }

    pub fn is_double(&self) -> bool {
        unsafe {
            ffi::Dart_IsDouble(self.handle)
        }
    }

    pub fn is_boolean(&self) -> bool {
        unsafe {
            ffi::Dart_IsBoolean(self.handle)
        }
    }

    pub fn is_string(&self) -> bool {
        unsafe {
            ffi::Dart_IsString(self.handle)
        }
    }

    pub fn is_string_latin1(&self) -> bool {
        unsafe {
            ffi::Dart_IsStringLatin1(self.handle)
        }
    }

    pub fn is_external_string(&self) -> bool {
        unsafe {
            ffi::Dart_IsExternalString(self.handle)
        }
    }

    pub fn is_list(&self) -> bool {
        unsafe {
            ffi::Dart_IsList(self.handle)
        }
    }

    pub fn is_map(&self) -> bool {
        unsafe {
            ffi::Dart_IsMap(self.handle)
        }
    }

    pub fn is_library(&self) -> bool {
        unsafe {
            ffi::Dart_IsLibrary(self.handle)
        }
    }

    pub fn is_type(&self) -> bool {
        unsafe {
            ffi::Dart_IsType(self.handle)
        }
    }

    pub fn is_function(&self) -> bool {
        unsafe {
            ffi::Dart_IsFunction(self.handle)
        }
    }

    pub fn is_variable(&self) -> bool {
        unsafe {
            ffi::Dart_IsVariable(self.handle)
        }
    }

    pub fn is_type_variable(&self) -> bool {
        unsafe {
            ffi::Dart_IsTypeVariable(self.handle)
        }
    }

    pub fn is_closure(&self) -> bool {
        unsafe {
            ffi::Dart_IsClosure(self.handle)
        }
    }

    pub fn is_typed_data(&self) -> bool {
        unsafe {
            ffi::Dart_IsTypedData(self.handle)
        }
    }

    pub fn is_byte_buffer(&self) -> bool {
        unsafe {
            ffi::Dart_IsByteBuffer(self.handle)
        }
    }

    pub fn is_future(&self) -> bool {
        unsafe {
            ffi::Dart_IsFuture(self.handle)
        }
    }

    pub fn get_instance_type(&self) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_InstanceGetType(self.handle);
            Self::new(handle).get_error()
        }
    }

    pub fn get_class_name(&self) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_ClassName(self.handle);
            Self::new(handle).get_error()
        }
    }

    pub fn get_function_name(&self) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_FunctionName(self.handle);
            Self::new(handle).get_error()
        }
    }

    pub fn get_function_owner(&self) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_FunctionOwner(self.handle);
            Self::new(handle).get_error()
        }
    }

    pub fn function_is_static(&self) -> Result<bool, Error> {
        unsafe {
            let mut result = MaybeUninit::uninit();
            let error_handle = ffi::Dart_FunctionIsStatic(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn is_tear_off(&self) -> bool {
        unsafe {
            ffi::Dart_IsTearOff(self.handle)
        }
    }

    pub fn function_from_closure(&self) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_ClosureFunction(self.handle);
            Self::new(handle).get_error()
        }
    }

    pub fn library_from_class(&self) -> Result<Self, Error> {
        unsafe {
            let handle = ffi::Dart_ClassLibrary(self.handle);
            Self::new(handle).get_error()
        }
    }

    pub fn integer_fits_in_i64(&self) -> Result<bool, Error> {
        unsafe {
            let mut result = MaybeUninit::uninit();
            let error_handle = ffi::Dart_IntegerFitsIntoInt64(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn integer_fits_in_u64(&self) -> Result<bool, Error> {
        unsafe {
            let mut result = MaybeUninit::uninit();
            let error_handle = ffi::Dart_IntegerFitsIntoUint64(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn new_i64(x: i64) -> Self {
        unsafe {
            Self::new(ffi::Dart_NewInteger(x))
        }
    }

    pub fn new_u64(x: u64) -> Self {
        unsafe {
            Self::new(ffi::Dart_NewIntegerFromUint64(x))
        }
    }

    pub fn parse_hex_int(num: &CStr) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewIntegerFromHexCString(num.as_ptr())).get_error()
        }
    }

    pub fn get_i64(&self) -> Result<i64, Error> {
        unsafe {
            let mut result = MaybeUninit::<i64>::uninit();
            let error_handle = ffi::Dart_IntegerToInt64(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn get_u64(&self) -> Result<u64, Error> {
        unsafe {
            let mut result = MaybeUninit::<u64>::uninit();
            let error_handle = ffi::Dart_IntegerToUint64(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn get_integer_hex_string(&self) -> Result<CString, Error> {
        unsafe {
            let mut result = MaybeUninit::<*const c_char>::uninit();
            let error_handle = ffi::Dart_IntegerToHexCString(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            let cstr = CStr::from_ptr(result.assume_init());
            Ok(cstr.to_owned())
        }
    }

    pub fn new_f64(x: f64) -> Self {
        unsafe {
            Self::new(ffi::Dart_NewDouble(x))
        }
    }

    pub fn get_f64(&self) -> Result<f64, Error> {
        unsafe {
            let mut result = MaybeUninit::<f64>::uninit();
            let error_handle = ffi::Dart_DoubleValue(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn get_static_method_closure(library: Self, clazz: Self, function_name: Self) -> Result<Self, Error> {
        unsafe {
            Self::new(
                ffi::Dart_GetStaticMethodClosure(
                    library.handle,
                    clazz.handle,
                    function_name.handle)
            ).get_error()
        }
    }

    pub fn const_true() -> Self {
        unsafe {
            Self::new(ffi::Dart_True())
        }
    }

    pub fn const_false() -> Self {
        unsafe {
            Self::new(ffi::Dart_False())
        }
    }

    pub fn new_bool(x: bool) -> Self {
        unsafe {
            Self::new(ffi::Dart_NewBoolean(x))
        }
    }

    pub fn get_bool(&self) -> Result<bool, Error> {
        unsafe {
            let mut result = MaybeUninit::<bool>::uninit();
            let error_handle = ffi::Dart_BooleanValue(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init())
        }
    }

    pub fn string_length(&self) -> Result<usize, Error> {
        unsafe {
            let mut result = MaybeUninit::<isize>::uninit();
            let error_handle = ffi::Dart_StringLength(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init() as usize)
        }
    }

    pub fn string_from_cstr(string: &CStr) -> Self {
        unsafe {
            Self::new(ffi::Dart_NewStringFromCString(string.as_ptr()))
        }
    }

    pub fn string_from_str(string: &str) -> Self {
        unsafe {
            Self::new(ffi::Dart_NewStringFromUTF8(string.as_ptr(), string.len() as _))
        }
    }

    pub fn string_from_utf8(string: &[u8]) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewStringFromUTF8(string.as_ptr(), string.len() as _)).get_error()
        }
    }

    pub fn string_from_utf16(utf16: &[u16]) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewStringFromUTF16(utf16.as_ptr(), utf16.len() as _)).get_error()
        }
    }

    pub fn string_from_utf32(utf32: &[i32]) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewStringFromUTF32(utf32.as_ptr(), utf32.len() as _)).get_error()
        }
    }

    pub fn string_to_cstring(&self) -> Result<CString, Error> {
        unsafe {
            let mut result = MaybeUninit::<*const c_char>::uninit();
            let error_handle = ffi::Dart_StringToCString(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            let cstr = CStr::from_ptr(result.assume_init());
            Ok(cstr.to_owned())
        }
    }

    pub fn string_to_utf8(&self) -> Result<String, Error> {
        unsafe {
            let mut ptr = MaybeUninit::<*mut u8>::uninit();
            let mut len = MaybeUninit::<isize>::uninit();
            let error_handle = ffi::Dart_StringToUTF8(self.handle, ptr.as_mut_ptr(), len.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            let slice = std::slice::from_raw_parts_mut(ptr.assume_init(), len.assume_init() as _);
            let string = String::from_utf8_lossy(slice);
            Ok(string.into_owned())
        }
    }

    pub fn string_storage_size(&self) -> Result<usize, Error> {
        unsafe {
            let mut result = MaybeUninit::<isize>::uninit();
            let error_handle = ffi::Dart_StringStorageSize(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            let result = result.assume_init().try_into().unwrap();
            Ok(result)
        }
    }

    pub fn new_list(length: usize) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewList(length as _)).get_error()
        }
    }

    pub fn new_list_of(length: usize, ty: ffi::Dart_CoreType_Id) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewListOf(ty, length as _)).get_error()
        }
    }

    pub fn new_list_of_self_as_type(&self, length: usize) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewListOfType(self.handle, length as _)).get_error()
        }
    }

    pub fn list_length(&self) -> Result<usize, Error> {
        unsafe {
            let mut result = MaybeUninit::<isize>::uninit();
            let error_handle = ffi::Dart_ListLength(self.handle, result.as_mut_ptr());
            Self::new(error_handle).get_error()?;
            Ok(result.assume_init() as usize)
        }
    }

    pub fn list_at(&self, index: usize) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_ListGetAt(self.handle, index as _)).get_error()
        }
    }

    pub fn list_get_range(&self, range: impl std::ops::RangeBounds<usize>) -> Result<Self, Error> {
        use std::ops::Bound::*;
        let start = match range.start_bound() {
            Included(x) => *x,
            Excluded(x) => *x + 1,
            Unbounded => 0
        };
        let end = match range.end_bound() {
            Included(x) => *x + 1,
            Excluded(x) => *x,
            Unbounded => self.list_length()?,
        };
        let len = end - start;
        unsafe {
            let mut result = MaybeUninit::<ffi::Dart_Handle>::uninit();
            let error_handle =
                ffi::Dart_ListGetRange(
                    self.handle,
                    start as isize,
                    len as isize,
                    result.as_mut_ptr()
                );
            Self::new(error_handle)
                .get_error()
                .and_then(|_| Self::new(result.assume_init()).get_error())
        }
    }

    pub fn list_set_at(&self, item: Self, index: usize) -> Result<(), Error> {
        unsafe {
            Self::new(ffi::Dart_ListSetAt(self.handle, index as _, item.handle))
                .get_error()
                .map(|_| ())
        }
    }

    pub fn map_get_at(&self, key: Self) -> Result<Option<Self>, Error> {
        unsafe {
            let result = ffi::Dart_MapGetAt(self.handle, key.handle);
            Self::new(result)
                .get_error()
                .map(|x|
                    if x.is_null() {
                        None
                    } else {
                        Some(x)
                    }
                )
        }
    }

    pub fn map_contains_key(&self, key: Self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_MapContainsKey(self.handle, key.handle)).get_error()
        }
    }

    pub fn map_keys(&self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_MapKeys(self.handle)).get_error()
        }
    }

    pub fn typed_data_get_type(&self) -> ffi::Dart_TypedData_Type {
        unsafe {
            ffi::Dart_GetTypeOfTypedData(self.handle)
        }
    }

    pub fn external_typed_data_get_type(&self) -> ffi::Dart_TypedData_Type {
        unsafe {
            ffi::Dart_GetTypeOfExternalTypedData(self.handle)
        }
    }

    pub fn new_typed_data(ty: ffi::Dart_TypedData_Type, len: usize) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_NewTypedData(ty, len as _)).get_error()
        }
    }

    pub unsafe fn new_external_typed_data<T: TypedData>(values: *mut [T]) -> Result<Self, Error> {
        Self::new(
            ffi::Dart_NewExternalTypedData(T::TYPE, values as *mut _, <*const [T]>::as_ref(values).unwrap().len() as _)
        ).get_error()
    }

    pub fn new_external_typed_data_with_drop<T: TypedData, V: Into<Box<[T]>>>(values: V) -> Result<Self, Error> {
        let ptr = Box::leak(values.into());
        let len = ptr.len();
        let ptr_ptr = Box::leak(Box::new(ptr as *mut [T]));

        unsafe extern "C" fn free<T>(_isolate_callback_data: *mut c_void, _handle: ffi::Dart_WeakPersistentHandle, peer: *mut c_void) {
            let ptr = peer as *mut *mut [T];
            let boxed = Box::from_raw(*ptr);
            drop(boxed);
        }

        unsafe {
            let handle = ffi::Dart_NewExternalTypedDataWithFinalizer(
                T::TYPE,
                ptr.as_mut_ptr() as *mut _,
                len as isize,
                ptr_ptr as *mut *mut [T] as *mut _,
                (len * std::mem::size_of::<T>()) as _,
                Some(
                    free::<T>
                )
            );
            Self::new(handle).get_error()
        }
    }

    pub fn new_of_type_self(&self, constructor_name: Option<Self>, args: &mut [Self]) -> Result<Self, Error> {
        // SAFETY:
        // Self is `repr(transparent)`, so we can
        // directly pointer cast to the array of handles.
        unsafe {
            Self::new(
                ffi::Dart_New(
                    self.handle,
                    constructor_name.unwrap_or_else(Self::null).handle,
                    args.len() as i32,
                    args as *mut [Self] as *mut [ffi::Dart_Handle] as *mut _
                )
            ).get_error()
        }
    }

    pub fn allocate_of_type_self(&self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_Allocate(self.handle)).get_error()
        }
    }

    pub fn invoke(&self, function_name: Self, args: &mut [Self]) -> Result<Self, Error> {
        // SAFETY:
        // Self is `repr(transparent)`, so we can
        // directly pointer cast to the array of handles.
        unsafe {
            Self::new(
                ffi::Dart_Invoke(
                    self.handle,
                    function_name.handle,
                    args.len() as i32,
                    args as *mut [Self] as *mut [ffi::Dart_Handle] as *mut _
                )
            ).get_error()
        }
    }

    pub fn invoke_closure(&self, args: &mut [Self]) -> Result<Self, Error> {
        // SAFETY:
        // Self is `repr(transparent)`, so we can
        // directly pointer cast to the array of handles.
        unsafe {
            Self::new(
                ffi::Dart_InvokeClosure(
                    self.handle,
                    args.len() as i32,
                    args as *mut [Self] as *mut [ffi::Dart_Handle] as *mut _
                )
            ).get_error()
        }
    }

    pub fn invoke_self_constructor(&self, name: Option<Self>, args: &mut [Self]) -> Result<Self, Error> {
        // SAFETY:
        // Self is `repr(transparent)`, so we can
        // directly pointer cast to the array of handles.
        unsafe {
            Self::new(
                ffi::Dart_InvokeConstructor(
                    self.handle,
                    name.unwrap_or_else(Self::null).handle,
                    args.len() as i32,
                    args as *mut [Self] as *mut [ffi::Dart_Handle] as *mut _
                )
            ).get_error()
        }
    }

    pub fn get_field(&self, name: Self) -> Result<Self, Error> {
        unsafe {
            Self::new(
                ffi::Dart_GetField(self.handle, name.handle)
            ).get_error()
        }
    }

    pub fn set_field(&self, name: Self, value: Self) -> Result<(), Error> {
        unsafe {
            Self::new(
                ffi::Dart_SetField(self.handle, name.handle, value.handle)
            ).get_error()?;
            Ok(())
        }
    }

    pub fn make_type_from_decl(library: Self, class_name: Self, type_args: &mut [Self]) -> Result<Self, Error> {
        unsafe {
            Self::new(
                ffi::Dart_GetType(
                    library.handle,
                    class_name.handle,
                    type_args.len() as _,
                    type_args as *mut [Self] as *mut [ffi::Dart_Handle] as *mut ffi::Dart_Handle
                )
            ).get_error()
        }
    }

    pub fn get_class_of_library(library: Self, name: Self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_GetClass(library.handle, name.handle)).get_error()
        }
    }

    pub fn get_library_url_import(&self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_LibraryUrl(self.handle)).get_error()
        }
    }

    pub fn get_library_url_path(&self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_LibraryResolvedUrl(self.handle)).get_error()
        }
    }

    pub fn get_loaded_libraries(&self) -> Result<Self, Error> {
        unsafe {
            Self::new(ffi::Dart_GetLoadedLibraries()).get_error()
        }
    }

    pub fn op_add(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("+"), &mut [other])
    }

    pub fn op_sub(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("-"), &mut [other])
    }

    pub fn op_mul(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("*"), &mut [other])
    }

    pub fn op_div(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("/"), &mut [other])
    }

    pub fn op_rem(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("~/"), &mut [other])
    }

    pub fn op_neg(&self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("unary-"), &mut [])
    }

    pub fn op_eq(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("=="), &mut [other])
    }

    pub fn op_gt(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str(">"), &mut [other])
    }

    pub fn op_gte(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str(">="), &mut [other])
    }

    pub fn op_lt(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("<"), &mut [other])
    }

    pub fn op_lte(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("<="), &mut [other])
    }

    pub fn op_bitand(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("&"), &mut [other])
    }

    pub fn op_bitor(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("|"), &mut [other])
    }

    pub fn op_bitxor(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("^"), &mut [other])
    }

    pub fn op_bit_not(&self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("~"), &mut [])
    }

    pub fn op_shl(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("<<"), &mut [other])
    }

    pub fn op_shr(&self, other: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str(">>"), &mut [other])
    }

    pub fn op_idx(&self, idx: Self) -> Result<Self, Error> {
        self.invoke(Self::string_from_str("[]"), &mut [idx])
    }

    pub fn op_idx_assign(&self, idx: Self, value: Self) -> Result<(), Error> {
        self.invoke(Self::string_from_str("[]="), &mut [idx, value]).map(drop)
    }
}

impl Deref for UnverifiedDartHandle {
    type Target = ffi::Dart_Handle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Debug for UnverifiedDartHandle {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{:?}", self.handle)
    }
}

impl PartialEq<Self> for UnverifiedDartHandle {
    fn eq(&self, other: &Self) -> bool {
        UnverifiedDartHandle::identity_eq(*self, *other)
    }
}

mod impl_ops_unverified_dart_handle {
    use crate::dart_unwrap;
    use super::{UnverifiedDartHandle, Error};
    use std::ops::{
        Add, AddAssign,
        Sub, SubAssign,
        Mul, MulAssign,
        Div, DivAssign,
        Rem, RemAssign,
        Neg, Not,
        BitAnd, BitAndAssign,
        BitOr, BitOrAssign,
        BitXor, BitXorAssign,
        Shl, ShlAssign,
        Shr, ShrAssign,
    };

    impl Neg for UnverifiedDartHandle {
        type Output = Result<UnverifiedDartHandle, Error>;
        #[inline]
        fn neg(self) -> Result<UnverifiedDartHandle, Error> {
            self.op_neg()
        }
    }

    impl Neg for &UnverifiedDartHandle {
        type Output = Result<UnverifiedDartHandle, Error>;
        #[inline]
        fn neg(self) -> Result<UnverifiedDartHandle, Error> {
            self.op_neg()
        }
    }

    impl Not for UnverifiedDartHandle {
        type Output = Result<UnverifiedDartHandle, Error>;
        #[inline]
        fn not(self) -> Result<UnverifiedDartHandle, Error> {
            self.op_bit_not()
        }
    }

    impl Not for &UnverifiedDartHandle {
        type Output = Result<UnverifiedDartHandle, Error>;
        #[inline]
        fn not(self) -> Result<UnverifiedDartHandle, Error> {
            self.op_bit_not()
        }
    }

    macro_rules! impl_ops_unverified_dart_handle {
        ($($simple:ident, $simple_op:ident, $assign:ident, $assign_op:ident, $op:ident,)*) => {
            $(
                impl $simple<UnverifiedDartHandle> for UnverifiedDartHandle {
                    type Output = Result<Self, Error>;
                    #[inline]
                    fn $simple_op(self, rhs: UnverifiedDartHandle) -> Result<Self, Error> {
                        self.$op(rhs)
                    }
                }

                impl $simple<&UnverifiedDartHandle> for UnverifiedDartHandle {
                    type Output = Result<Self, Error>;
                    #[inline]
                    fn $simple_op(self, rhs: &UnverifiedDartHandle) -> Result<Self, Error> {
                        self.$op(*rhs)
                    }
                }

                impl $simple<UnverifiedDartHandle> for &UnverifiedDartHandle {
                    type Output = Result<UnverifiedDartHandle, Error>;
                    #[inline]
                    fn $simple_op(self, rhs: UnverifiedDartHandle) -> Result<UnverifiedDartHandle, Error> {
                        self.$op(rhs)
                    }
                }

                impl $simple<&UnverifiedDartHandle> for &UnverifiedDartHandle {
                    type Output = Result<UnverifiedDartHandle, Error>;
                    #[inline]
                    fn $simple_op(self, rhs: &UnverifiedDartHandle) -> Result<UnverifiedDartHandle, Error> {
                        self.$op(*rhs)
                    }
                }

                impl $assign<UnverifiedDartHandle> for UnverifiedDartHandle {
                    #[inline]
                    fn $assign_op(&mut self, rhs: UnverifiedDartHandle) {
                        *self = dart_unwrap!(self.$op(rhs));
                    }
                }

                impl $assign<&UnverifiedDartHandle> for UnverifiedDartHandle {
                    #[inline]
                    fn $assign_op(&mut self, rhs: &UnverifiedDartHandle) {
                        *self = dart_unwrap!(self.$op(*rhs));
                    }
                }
            )*
        }
    }

    impl_ops_unverified_dart_handle!(
        Add, add, AddAssign, add_assign, op_add,
        Sub, sub, SubAssign, sub_assign, op_sub,
        Mul, mul, MulAssign, mul_assign, op_mul,
        Div, div, DivAssign, div_assign, op_div,
        Rem, rem, RemAssign, rem_assign, op_rem,
        BitAnd, bitand, BitAndAssign, bitand_assign, op_bitand,
        BitOr, bitor, BitOrAssign, bitor_assign, op_bitor,
        BitXor, bitxor, BitXorAssign, bitxor_assign, op_bitxor,
        Shl, shl, ShlAssign, shl_assign, op_shl,
        Shr, shr, ShrAssign, shr_assign, op_shr,
    );
}

pub fn version_string() -> CString {
    unsafe {
        let ptr = ffi::Dart_VersionString();
        let cstr = CStr::from_ptr(ptr);
        cstr.to_owned()
    }
}

pub struct Error {
    handle: UnverifiedDartHandle,
    kind: ErrorKind
}

unsafe impl DartHandle for Error {
    fn handle(&self) -> ffi::Dart_Handle {
        self.handle.handle
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        match handle.get_error() {
            Ok(x) => Err(x),
            Err(y) => Ok(y),
        }
    }
}

impl Error {
    pub(crate) unsafe fn of(handle: UnverifiedDartHandle, kind: ErrorKind) -> Self {
        Self {
            handle,
            kind
        }
    }

    pub fn get_msg(&self) -> CString {
        unsafe {
            let ptr = ffi::Dart_GetError(*self.handle);
            let cstr = CStr::from_ptr(ptr);
            cstr.to_owned()
        }
    }

    pub fn is_exception(&self) -> bool {
        unsafe {
            ffi::Dart_ErrorHasException(*self.handle)
        }
    }

    pub fn get_exception(&self) -> Option<UnverifiedDartHandle> {
        if let ErrorKind::UnhandledException = self.kind {
            unsafe {
                Some(UnverifiedDartHandle::new(ffi::Dart_ErrorGetException(*self.handle)))
            }
        } else {
            None
        }
    }

    pub fn get_stack_trace(&self) -> Option<UnverifiedDartHandle> {
        if let ErrorKind::UnhandledException = self.kind {
            unsafe {
                Some(UnverifiedDartHandle::new(ffi::Dart_ErrorGetStackTrace(*self.handle)))
            }
        } else {
            None
        }
    }

    pub fn new_api(message: &str) -> Result<Error, NulError> {
        let cstring = CString::new(message)?;
        unsafe {
            Ok(
                Self {
                    handle: UnverifiedDartHandle::new(
                        ffi::Dart_NewApiError(cstring.as_ptr()),
                    ),
                    kind: ErrorKind::Api
                }
            )
        }
    }

    pub fn new_compilation(message: &str) -> Result<Error, NulError> {
        let cstring = CString::new(message)?;
        unsafe {
            Ok(
                Self {
                    handle: UnverifiedDartHandle::new(
                        ffi::Dart_NewCompilationError(cstring.as_ptr()),
                    ),
                    kind: ErrorKind::Compilation,
                }
            )
        }
    }

    pub fn new_unhandled_exception(exception: UnverifiedDartHandle) -> Error {
        unsafe {
            Self {
                handle: UnverifiedDartHandle::new(
                    ffi::Dart_NewUnhandledExceptionError(*exception),
                ),
                kind: ErrorKind::UnhandledException,
            }
        }
    }

    pub fn propagate_error(self) -> Infallible {
        unsafe {
            ffi::Dart_PropagateError(*self.handle);
        }
        panic!("This should not happen!");
    }

    pub fn throw_self(self) -> Result<Infallible, Error> {
        let handle = unsafe {
            UnverifiedDartHandle::new(ffi::Dart_ThrowException(self.handle())).get_error()
        };

        handle?;
        panic!("Reached a non error handle after throwing an Exception!");
    }

    pub fn rethrow_self(self, stacktrace: UnverifiedDartHandle) -> Result<Infallible, Error> {
        let handle = unsafe {
            UnverifiedDartHandle::new(ffi::Dart_ReThrowException(self.handle(), stacktrace.handle)).get_error()
        };

        handle?;
        panic!("Reached a non error handle after rethrowing an Exception!");
    }
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self.handle.to_string().unwrap())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ErrorKind {
    Api,
    UnhandledException,
    Compilation,
    Fatal,
}

unsafe impl DartHandle for Result<UnverifiedDartHandle, Error> {
    fn handle(&self) -> ffi::Dart_Handle {
        match self {
            Ok(x) => x.handle,
            Err(e) => e.handle.handle,
        }
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        match self {
            Ok(x) => *x,
            Err(e) => e.safe_handle()
        }
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        Ok(handle.get_error())
    }
}

pub struct Port {
    pub(crate) port: ffi::Dart_Port
}

impl Port {
    pub unsafe fn from_port(port: ffi::Dart_Port) -> Option<Self> {
        if port == ffi::ILLEGAL_PORT {
            None
        } else {
            Some(Self { port })
        }
    }

    pub fn post<T: DartHandle>(&self, handle: T) -> bool {
        unsafe {
            ffi::Dart_Post(self.port, handle.handle())
        }
    }
    pub unsafe fn post_cobject(&self, obj: &mut Dart_CObject) -> bool {
        ffi::Dart_PostCObject(self.port, obj)
    }
    pub fn post_integer(&self, num: i64) -> bool {
        unsafe {
            ffi::Dart_PostInteger(self.port, num)
        }
    }

    pub unsafe fn new(port: ffi::Dart_Port) -> Result<(Self, UnverifiedDartHandle), Error> {
        let handle = ffi::Dart_NewSendPort(port);
        let handle = UnverifiedDartHandle::new(handle).get_error()?;
        Ok((
            Self { port },
            handle,
        ))
    }
    pub fn get_main_port() -> Self {
        Self {
            port: unsafe {
                ffi::Dart_GetMainPortId()
            }
        }
    }
    pub fn from_send_port(handle: UnverifiedDartHandle) -> Result<Self, Error> {
        let mut port = MaybeUninit::uninit();
        let error_handle = unsafe {ffi::Dart_SendPortGetId(handle.handle, port.as_mut_ptr())};
        unsafe {
            UnverifiedDartHandle::new(error_handle).get_error()?;
            Ok(Self {
                port: port.assume_init()
            })
        }
    }
}

pub struct NativePort {
    port: Port,
}

impl NativePort {
    pub unsafe fn new_native(name: CString, handler: unsafe extern "C" fn(dest_port_id: ffi::Dart_Port, message: *mut ffi::Dart_CObject)) -> Option<Self> {
        let port = ffi::Dart_NewNativePort(
            name.as_ptr(),
            Some(handler),
            true // handle_concurrently will always be true since rust is awesome about concurrency.
        );
        let port = Port::from_port(port)?;
        Some(
            Self {
                port
            }
        )
    }

    pub fn close(self) -> bool {
        unsafe {
            ffi::Dart_CloseNativePort(self.port.port)
        }
    }

    pub fn port(&self) -> ffi::Dart_Port {
        self.port.port
    }
}

pub trait TypedData: 'static + Copy + Clone + Debug {
    const TYPE: ffi::Dart_TypedData_Type;
}

macro_rules! impl_typed_data {
    ($($t:ty, $T:ident),*) => {
        $(
            impl TypedData for $t {
                const TYPE: ffi::Dart_TypedData_Type = ffi::Dart_TypedData_Type::$T;
            }
        )*
    }
}

impl_typed_data!(
    u8, Uint8,
    i8, Int8,
    u16, Uint16,
    i16, Int16,
    u32, Uint32,
    i32, Int32,
    u64, Uint64,
    i64, Int64,
    f32, Float32,
    f64, Float64
);

pub unsafe fn set_thread_name(name: &CStr) {
    ffi::Dart_SetThreadName(name.as_ptr());
}

pub unsafe fn enter_scope() {
    ffi::Dart_EnterScope();
}

pub unsafe fn exit_scope() {
    ffi::Dart_ExitScope();
}
