use crate::dart_unwrap;
use crate::dart_handle::{UnverifiedDartHandle, Error, DartHandle};
use std::ops::{Deref, RangeBounds, Mul, Add};
use super::integer::Integer;
use crate::dart_types::DartType;
use std::thread::LocalKey;

#[derive(Clone, Copy)]
pub struct DString {
    handle: UnverifiedDartHandle,
}

// Rust Equivalent Implementation
impl DString {
    pub fn new(string: &str) -> Self {
        Self {
            handle: UnverifiedDartHandle::string_from_str(string),
        }
    }

    pub fn from_utf8(bytes: &[u8]) -> Result<Self, Error> {
        Ok(
            Self {
                handle: UnverifiedDartHandle::string_from_utf8(bytes)?
            }
        )
    }

    pub fn from_utf16(values: &[u16]) -> Result<Self, Error> {
        Ok(
            Self {
                handle: UnverifiedDartHandle::string_from_utf16(values)?
            }
        )
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.as_string().into_bytes()
    }

    pub fn empty() -> Self {
        Self {
            handle: UnverifiedDartHandle::empty_string()
        }
    }

    pub fn index(&self, idx: usize) -> u16 {
        let idx = Integer::from(idx);
        let num = dart_unwrap!(self.handle.invoke(
            *Self::new("codeUnitAt"),
            &mut [idx.safe_handle()],
        ));
        Integer::from_handle(num).unwrap().value() as _
    }

    pub fn as_string(&self) -> String {
        dart_unwrap!(self.handle.string_to_utf8())
    }
}

thread_local! {
    #[allow(non_upper_case_globals)]
    pub static StringType: UnverifiedDartHandle = {
        let empty_string = UnverifiedDartHandle::empty_string();
        empty_string.get_instance_type().unwrap()
    };
}

//Dart Equivalent Implementation
impl DString {
    pub fn from_char_code(code: Integer) -> Self {
        let result = StringType.with(|x| {
            x.new_of_type_self(Some(UnverifiedDartHandle::string_from_str("fromCharCode")), &mut [*code])
        });
        Self::from_handle(dart_unwrap!(result)).ok().unwrap()
    }

    pub fn from_char_codes(_codes: impl DartHandle) -> Self {
        unimplemented!()
    }

    pub fn from_environment(name: &Self, default: Option<&Self>) -> Self {
        let default = default.map(|x| x.handle).unwrap_or_else(UnverifiedDartHandle::null);


        let result = StringType.with(|x| {
            x.new_of_type_self(Some(UnverifiedDartHandle::string_from_str("fromEnvironment")), &mut [**name, default])
        });
        Self::from_handle(dart_unwrap!(result)).ok().unwrap()

    }

    pub fn code_units(&self) {
        unimplemented!()
    }

    pub fn hash_code(&self) -> Integer {
        Integer::from_handle(
            dart_unwrap!(self.handle.invoke(UnverifiedDartHandle::string_from_str("hashCode"), &mut []))
        ).ok().unwrap()
    }

    pub fn is_empty(&self) {
        loop {
            unimplemented!()
        }
    }

    pub fn is_not_empty(&self) {
        loop {
            unimplemented!()
        }
    }

    pub fn len(&self) -> usize {
        let result = self.handle.string_length();
        dart_unwrap!(result)
    }

    pub fn length(&self) -> Integer {
        Integer::from_handle(
            dart_unwrap!(self.handle.invoke(UnverifiedDartHandle::string_from_str("length"), &mut []))
        ).ok().unwrap()
    }

    pub fn runes(&self) -> impl DartHandle {
        let handle = self.handle.invoke(UnverifiedDartHandle::string_from_str("runes"), &mut []);
        dart_unwrap!(handle)
    }

    pub fn code_unit_at(&self, idx: Integer) -> Result<Integer, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("codeUnitAt"),
                &mut [idx.safe_handle()]
            )
            .map(
                |x| Integer::from_handle(x).ok().unwrap()
            )
    }

    pub fn compare_to(&self, other: DString) -> Integer {
        let handle = self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("compareTo"),
                &mut [other.safe_handle()]
            );
        Integer::from_handle(dart_unwrap!(handle)).ok().unwrap()
    }

    pub fn contains(&self, _pattern: impl DartHandle, _start_index: Option<Integer>) {
        loop {
            unimplemented!()
        }
    }

    pub fn ends_with(&self, _other: Self) {
        loop {
            unimplemented!()
        }
    }

    pub fn index_of(&self, pattern: impl DartHandle, start: Option<Integer>) -> Result<Integer, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("indexOf"),
                &mut [
                    pattern.safe_handle(),
                    start.map(|x| x.safe_handle()).unwrap_or_else(UnverifiedDartHandle::null)
                ]
            )
            .map(
                |x| Integer::from_handle(x).ok().unwrap()
            )
    }

    pub fn last_index_of(&self, pattern: impl DartHandle, start: Option<Integer>) -> Result<Integer, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("lastIndexOf"),
                &mut [
                    pattern.safe_handle(),
                    start.map(|x| x.safe_handle()).unwrap_or_else(UnverifiedDartHandle::null)
                ]
            )
            .map(
                |x| Integer::from_handle(x).ok().unwrap()
            )
    }

    pub fn pad_left(&self, width: Integer, padding: Option<Self>) -> Result<Self, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("padLeft"),
                &mut [
                    width.safe_handle(),
                    padding.unwrap_or_else(|| Self::new(" ")).safe_handle()
                ]
            )
            .map(
                |x| Self::from_handle(x).ok().unwrap()
            )
    }

    pub fn pad_right(&self, width: Integer, padding: Option<Self>) -> Result<Self, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("padRight"),
                &mut [
                    width.safe_handle(),
                    padding.unwrap_or_else(|| Self::new(" ")).safe_handle()
                ]
            )
            .map(
                |x| Self::from_handle(x).ok().unwrap()
            )
    }

    pub fn replace_all(&self, from: impl DartHandle, replace: Self) -> Result<Self, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("replaceAll"),
                &mut [
                    from.safe_handle(),
                    replace.safe_handle()
                ]
            )
            .map(|x| Self::from_handle(x).ok().unwrap())
    }

    //Not going to be supported yet.
    pub fn replace_all_mapped(&self, _from: impl DartHandle, _replace: impl DartHandle) {
        unimplemented!()
    }

    pub fn replace_first(&self, from: impl DartHandle, to: Self, start_index: Option<Integer>) -> Result<Self, Error> {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("replaceFirst"),
                &mut [
                    from.safe_handle(),
                    to.safe_handle(),
                    start_index.unwrap_or_else(|| Integer::new(0)).safe_handle(),
                ]
            )
            .map(|x| Self::from_handle(x).ok().unwrap())
    }

    //Not going to be supported yet.
    pub fn replace_first_mapped(&self, _from: impl DartHandle, _replace: impl DartHandle, _start_index: Option<Integer>) {
        unimplemented!()
    }

    pub fn replace_range(&self, range: impl RangeBounds<Integer>, replacement: Self) -> Result<Self, Error> {
        let start = match range.start_bound() {
            std::ops::Bound::Excluded(_) | std::ops::Bound::Unbounded => panic!("Unbounded starts are not supported!"),
            std::ops::Bound::Included(x) => x.safe_handle()
        };
        let end = match range.end_bound() {
            std::ops::Bound::Unbounded => UnverifiedDartHandle::null(),
            std::ops::Bound::Excluded(x) => x.safe_handle(),
            std::ops::Bound::Included(x) => Integer::from(x + 1).safe_handle()
        };
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("replaceRange"),
                &mut [
                    start,
                    end,
                    replacement.safe_handle(),
                ]
            )
            .map(
                |x| Self::from_handle(x).ok().unwrap()
            )
    }

    pub fn split(&self, _pattern: impl DartHandle) {
        unimplemented!()
    }

    //Not going to be supported yet.
    pub fn split_map_join(&self, _pattern: impl DartHandle, _on_match: impl DartHandle, _on_non_match: impl DartHandle) {
        unimplemented!()
    }

    pub fn starts_with(&self, _pattern: impl DartHandle) {
        unimplemented!()
    }

    pub fn substring(&self, range: impl RangeBounds<Integer>) -> Result<Self, Error> {
        let start = match range.start_bound() {
            std::ops::Bound::Excluded(_) | std::ops::Bound::Unbounded => panic!("Unbounded starts are not supported!"),
            std::ops::Bound::Included(x) => x.safe_handle()
        };
        let end = match range.end_bound() {
            std::ops::Bound::Unbounded => UnverifiedDartHandle::null(),
            std::ops::Bound::Excluded(x) => x.safe_handle(),
            std::ops::Bound::Included(x) => Integer::from(x + 1).safe_handle()
        };
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("substring"),
                &mut [
                    start,
                    end,
                ]
            )
            .map(
                |x| Self::from_handle(x).ok().unwrap()
            )
    }

    pub fn to_lower_case(&self) -> Self {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("toLowerCase"),
                &mut []
            )
            .map(Self::from_handle).ok().unwrap().ok().unwrap()
    }

    pub fn to_upper_case(&self) -> Self {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("toUpperCase"),
                &mut []
            )
            .map(Self::from_handle).ok().unwrap().ok().unwrap()
    }

    pub fn trim(&self) -> Self {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("trim"),
                &mut []
            )
            .map(Self::from_handle).ok().unwrap().ok().unwrap()
    }

    pub fn trim_left(&self) -> Self {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("trimLeft"),
                &mut []
            )
            .map(Self::from_handle).ok().unwrap().ok().unwrap()
    }

    pub fn trim_right(&self) -> Self {
        self
            .handle
            .invoke(
                UnverifiedDartHandle::string_from_str("trimRight"),
                &mut []
            )
            .map(Self::from_handle).ok().unwrap().ok().unwrap()
    }

    pub fn mul_by(&self, times: Integer) -> Self {
        dart_unwrap!(self
            .handle
            .op_mul(times.safe_handle())
            .map(Self::from_handle))
            .ok()
            .unwrap()
    }

    pub fn add_to(&self, other: Self) -> Self {
        dart_unwrap!(self
            .handle
            .op_add(other.safe_handle())
            .map(Self::from_handle))
            .ok()
            .unwrap()
    }

    pub fn idx(&self, idx: Integer) -> Self {
        dart_unwrap!(self
            .handle
            .op_idx(idx.safe_handle())
            .map(Self::from_handle))
            .ok()
            .unwrap()
    }
}

impl Mul<Integer> for DString {
    type Output = DString;
    fn mul(self, other: Integer) -> Self {
        self.mul_by(other)
    }
}

impl Add<Self> for DString {
    type Output = DString;
    fn add(self, other: Self) -> Self {
        self.add_to(other)
    }
}

impl PartialEq<Self> for DString {
    fn eq(&self, other: &Self) -> bool {
        dart_unwrap!(self
            .handle
            .op_eq(other.safe_handle())
            .map(
                |x| x.get_bool()
            ))
            .ok()
            .unwrap()
    }
}

impl Deref for DString {
    type Target = UnverifiedDartHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl DartHandle for DString {
    fn handle(&self) -> dart_sys::Dart_Handle {
        self.handle.handle()
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        if handle.is_string() || handle.is_external_string() || handle.is_string_latin1() {
            Ok(Self {
                handle
            })
        } else {
            Err(handle)
        }
    }
}

impl DartType for DString {
    const THIS: &'static LocalKey<UnverifiedDartHandle> = &StringType;
}
