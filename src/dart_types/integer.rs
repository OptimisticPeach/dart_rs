use crate::dart_handle::{UnverifiedDartHandle, DartHandle};
use std::cell::Cell;
use crate::dart_unwrap;
use std::ops::Deref;
use crate::dart_types::DartType;
use std::thread::LocalKey;

#[derive(Clone, Debug)]
pub struct Integer {
    handle: UnverifiedDartHandle,
    value: Cell<Option<i64>>,
}

impl Integer {
    pub fn new(value: i64) -> Self {
        let handle = UnverifiedDartHandle::new_i64(value);
        Self {
            handle,
            value: Cell::new(Some(value))
        }
    }

    #[inline]
    pub fn value(&self) -> i64 {
        if let Some(x) = self.value.get() {
            x
        } else {
            let value = dart_unwrap!(self.handle.get_i64());
            self.value.set(Some(value));
            value
        }
    }

    pub fn to_hex_string(&self) -> String {
        dart_unwrap!(self.handle.get_integer_hex_string()).into_string().unwrap()
    }
}

mod impls {
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
    use super::Integer;
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

    impl PartialEq<Self> for Integer {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.value() == other.value()
        }
    }

    impl PartialEq<i64> for Integer {
        #[inline]
        fn eq(&self, other: &i64) -> bool {
            self.value() == *other
        }
    }

    impl PartialEq<Integer> for i64 {
        #[inline]
        fn eq(&self, other: &Integer) -> bool {
            *self == other.value()
        }
    }

    impl PartialOrd<Self> for Integer {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.value().partial_cmp(&other.value())
        }
    }

    impl PartialOrd<i64> for Integer {
        #[inline]
        fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
            self.value().partial_cmp(other)
        }
    }

    impl PartialOrd<Integer> for i64 {
        #[inline]
        fn partial_cmp(&self, other: &Integer) -> Option<std::cmp::Ordering> {
            self.partial_cmp(&other.value())
        }
    }

    impl Add<Self> for Integer {
        type Output = i64;
        #[inline]
        fn add(self, rhs: Self) -> i64 {
            self.value() + rhs.value()
        }
    }

    impl Add<i64> for Integer {
        type Output = i64;
        #[inline]
        fn add(self, rhs: i64) -> i64 {
            self.value() + rhs
        }
    }

    impl Add<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn add(self, rhs: Integer) -> i64 {
            rhs.value() + self
        }
    }

    impl AddAssign<Integer> for i64 {
        #[inline]
        fn add_assign(&mut self, rhs: Integer) {
            *self += rhs.value();
        }
    }

    impl Sub<Self> for Integer {
        type Output = i64;
        #[inline]
        fn sub(self, rhs: Self) -> i64 {
            self.value() - rhs.value()
        }
    }

    impl Sub<i64> for Integer {
        type Output = i64;
        #[inline]
        fn sub(self, rhs: i64) -> i64 {
            self.value() - rhs
        }
    }

    impl Sub<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn sub(self, rhs: Integer) -> i64 {
            self - rhs.value()
        }
    }

    impl SubAssign<Integer> for i64 {
        #[inline]
        fn sub_assign(&mut self, rhs: Integer) {
            *self -= rhs.value();
        }
    }

    impl Mul<Self> for Integer {
        type Output = i64;
        #[inline]
        fn mul(self, rhs: Self) -> i64 {
            self.value() + rhs.value()
        }
    }

    impl Mul<i64> for Integer {
        type Output = i64;
        #[inline]
        fn mul(self, rhs: i64) -> i64 {
            self.value() * rhs
        }
    }

    impl Mul<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn mul(self, rhs: Integer) -> i64 {
            rhs.value() * self
        }
    }

    impl MulAssign<Integer> for i64 {
        #[inline]
        fn mul_assign(&mut self, rhs: Integer) {
            *self *= rhs.value();
        }
    }

    impl Div<Self> for Integer {
        type Output = i64;
        #[inline]
        fn div(self, rhs: Self) -> i64 {
            self.value() / rhs.value()
        }
    }

    impl Div<i64> for Integer {
        type Output = i64;
        #[inline]
        fn div(self, rhs: i64) -> i64 {
            self.value() / rhs
        }
    }

    impl Div<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn div(self, rhs: Integer) -> i64 {
            self / rhs.value()
        }
    }

    impl DivAssign<Integer> for i64 {
        #[inline]
        fn div_assign(&mut self, rhs: Integer) {
            *self /= rhs.value();
        }
    }

    impl Rem<Self> for Integer {
        type Output = i64;
        #[inline]
        fn rem(self, rhs: Self) -> i64 {
            self.value() % rhs.value()
        }
    }

    impl Rem<i64> for Integer {
        type Output = i64;
        #[inline]
        fn rem(self, rhs: i64) -> i64 {
            self.value() % rhs
        }
    }

    impl Rem<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn rem(self, rhs: Integer) -> i64 {
            self % rhs.value()
        }
    }

    impl RemAssign<Integer> for i64 {
        #[inline]
        fn rem_assign(&mut self, rhs: Integer) {
            *self %= rhs.value();
        }
    }

    impl Neg for Integer {
        type Output = i64;
        #[inline]
        fn neg(self) -> i64 {
            -self.value()
        }
    }

    impl Not for Integer {
        type Output = i64;
        #[inline]
        fn not(self) -> i64 {
            !self.value()
        }
    }

    impl BitAnd<Self> for Integer {
        type Output = i64;
        #[inline]
        fn bitand(self, rhs: Self) -> i64 {
            self.value() & rhs.value()
        }
    }

    impl BitAnd<i64> for Integer {
        type Output = i64;
        #[inline]
        fn bitand(self, rhs: i64) -> i64 {
            self.value() & rhs
        }
    }

    impl BitAnd<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn bitand(self, rhs: Integer) -> i64 {
            self & rhs.value()
        }
    }

    impl BitAndAssign<Integer> for i64 {
        #[inline]
        fn bitand_assign(&mut self, rhs: Integer) {
            *self &= rhs.value();
        }
    }

    impl BitOr<Self> for Integer {
        type Output = i64;
        #[inline]
        fn bitor(self, rhs: Self) -> i64 {
            self.value() | rhs.value()
        }
    }

    impl BitOr<i64> for Integer {
        type Output = i64;
        #[inline]
        fn bitor(self, rhs: i64) -> i64 {
            self.value() | rhs
        }
    }

    impl BitOr<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn bitor(self, rhs: Integer) -> i64 {
            self | rhs.value()
        }
    }

    impl BitOrAssign<Integer> for i64 {
        #[inline]
        fn bitor_assign(&mut self, rhs: Integer) {
            *self |= rhs.value();
        }
    }

    impl BitXor<Self> for Integer {
        type Output = i64;
        #[inline]
        fn bitxor(self, rhs: Self) -> i64 {
            self.value() ^ rhs.value()
        }
    }

    impl BitXor<i64> for Integer {
        type Output = i64;
        #[inline]
        fn bitxor(self, rhs: i64) -> i64 {
            self.value() ^ rhs
        }
    }

    impl BitXor<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn bitxor(self, rhs: Integer) -> i64 {
            self ^ rhs.value()
        }
    }

    impl BitXorAssign<Integer> for i64 {
        #[inline]
        fn bitxor_assign(&mut self, rhs: Integer) {
            *self ^= rhs.value();
        }
    }

    impl Shl<Self> for Integer {
        type Output = i64;
        #[inline]
        fn shl(self, rhs: Self) -> i64 {
            self.value() << rhs.value()
        }
    }

    impl Shl<i64> for Integer {
        type Output = i64;
        #[inline]
        fn shl(self, rhs: i64) -> i64 {
            self.value() << rhs
        }
    }

    impl Shl<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn shl(self, rhs: Integer) -> i64 {
            self << rhs.value()
        }
    }

    impl ShlAssign<Integer> for i64 {
        #[inline]
        fn shl_assign(&mut self, rhs: Integer) {
            *self <<= rhs.value();
        }
    }

    impl Shr<Self> for Integer {
        type Output = i64;
        #[inline]
        fn shr(self, rhs: Self) -> i64 {
            self.value() >> rhs.value()
        }
    }

    impl Shr<i64> for Integer {
        type Output = i64;
        #[inline]
        fn shr(self, rhs: i64) -> i64 {
            self.value() >> rhs
        }
    }

    impl Shr<Integer> for i64 {
        type Output = i64;
        #[inline]
        fn shr(self, rhs: Integer) -> i64 {
            self >> rhs.value()
        }
    }

    impl ShrAssign<Integer> for i64 {
        #[inline]
        fn shr_assign(&mut self, rhs: Integer) {
            *self >>= rhs.value();
        }
    }

    macro_rules! impl_ref_ops {
        ($this:ty, $($op:ident, $f:ident),*) => {
            $(
                impl<T: Clone> $op<&'_ T> for $this
                where $this: $op<T> {
                    type Output = <$this as $op<T>>::Output;
                    #[inline]
                    fn $f(self, other: &T) -> Self::Output {
                        self.$f(other.clone())
                    }
                }

                impl<T: Clone> $op<T> for &'_ $this
                where $this: $op<T> {
                    type Output = <$this as $op<T>>::Output;
                    #[inline]
                    fn $f(self, other: T) -> Self::Output {
                        self.clone().$f(other)
                    }
                }
            )*
        }
    }

    impl_ref_ops!(Integer,
        Add, add,
        Sub, sub,
        Mul, mul,
        Div, div,
        Rem, rem,
        Shr, shr,
        Shl, shl,
        BitOr, bitor,
        BitXor, bitxor,
        BitAnd, bitand
    );

    impl_from!(i64, (Integer),
        u8, i8,
        u16, i16,
        u32, i32,
        u64, i64,
        u128, i128,
        usize, isize,
        f32, f64
    );
}

thread_local! {
    #[allow(non_upper_case_globals)]
    pub static IntegerType: UnverifiedDartHandle = {
        let zero = UnverifiedDartHandle::new_i64(0);
        zero.get_instance_type().unwrap()
    };
}

impl DartType for Integer {
    const THIS: &'static LocalKey<UnverifiedDartHandle> = &IntegerType;
}

impl Deref for Integer {
    type Target = UnverifiedDartHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl DartHandle for Integer {
    fn handle(&self) -> dart_sys::Dart_Handle {
        self.handle.handle()
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        if handle.is_integer() {
            Ok(Self {
                handle,
                value: Cell::new(None)
            })
        } else {
            Err(handle)
        }
    }
}
