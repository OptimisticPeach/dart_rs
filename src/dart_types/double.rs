use crate::dart_handle::{DartHandle, UnverifiedDartHandle};
use crate::dart_types::DartType;
use crate::dart_unwrap;
use std::cell::Cell;
use std::ops::Deref;
use std::thread::LocalKey;

#[derive(Clone, Debug)]
pub struct Double {
    handle: UnverifiedDartHandle,
    value: Cell<Option<f64>>,
}

impl Double {
    pub fn new(value: f64) -> Self {
        let handle = UnverifiedDartHandle::new_f64(value);
        Self {
            handle,
            value: Cell::new(Some(value)),
        }
    }

    #[inline]
    pub fn value(&self) -> f64 {
        if let Some(x) = self.value.get() {
            x
        } else {
            let value = dart_unwrap!(self.handle.get_f64());
            self.value.set(Some(value));
            value
        }
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
    use super::Double;
    use std::ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
    };

    impl PartialEq<Self> for Double {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.value() == other.value()
        }
    }

    impl PartialEq<f64> for Double {
        #[inline]
        fn eq(&self, other: &f64) -> bool {
            self.value() == *other
        }
    }

    impl PartialEq<Double> for f64 {
        #[inline]
        fn eq(&self, other: &Double) -> bool {
            *self == other.value()
        }
    }

    impl PartialOrd<Self> for Double {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.value().partial_cmp(&other.value())
        }
    }

    impl PartialOrd<f64> for Double {
        #[inline]
        fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
            self.value().partial_cmp(other)
        }
    }

    impl PartialOrd<Double> for f64 {
        #[inline]
        fn partial_cmp(&self, other: &Double) -> Option<std::cmp::Ordering> {
            self.partial_cmp(&other.value())
        }
    }

    impl Add<Self> for Double {
        type Output = f64;
        #[inline]
        fn add(self, rhs: Self) -> f64 {
            self.value() + rhs.value()
        }
    }

    impl Add<f64> for Double {
        type Output = f64;
        #[inline]
        fn add(self, rhs: f64) -> f64 {
            self.value() + rhs
        }
    }

    impl Add<Double> for f64 {
        type Output = f64;
        #[inline]
        fn add(self, rhs: Double) -> f64 {
            rhs.value() + self
        }
    }

    impl AddAssign<Double> for f64 {
        #[inline]
        fn add_assign(&mut self, rhs: Double) {
            *self += rhs.value();
        }
    }

    impl Sub<Self> for Double {
        type Output = f64;
        #[inline]
        fn sub(self, rhs: Self) -> f64 {
            self.value() - rhs.value()
        }
    }

    impl Sub<f64> for Double {
        type Output = f64;
        #[inline]
        fn sub(self, rhs: f64) -> f64 {
            self.value() - rhs
        }
    }

    impl Sub<Double> for f64 {
        type Output = f64;
        #[inline]
        fn sub(self, rhs: Double) -> f64 {
            self - rhs.value()
        }
    }

    impl SubAssign<Double> for f64 {
        #[inline]
        fn sub_assign(&mut self, rhs: Double) {
            *self -= rhs.value();
        }
    }

    impl Mul<Self> for Double {
        type Output = f64;
        #[inline]
        fn mul(self, rhs: Self) -> f64 {
            self.value() + rhs.value()
        }
    }

    impl Mul<f64> for Double {
        type Output = f64;
        #[inline]
        fn mul(self, rhs: f64) -> f64 {
            self.value() * rhs
        }
    }

    impl Mul<Double> for f64 {
        type Output = f64;
        #[inline]
        fn mul(self, rhs: Double) -> f64 {
            rhs.value() * self
        }
    }

    impl MulAssign<Double> for f64 {
        #[inline]
        fn mul_assign(&mut self, rhs: Double) {
            *self *= rhs.value();
        }
    }

    impl Div<Self> for Double {
        type Output = f64;
        #[inline]
        fn div(self, rhs: Self) -> f64 {
            self.value() / rhs.value()
        }
    }

    impl Div<f64> for Double {
        type Output = f64;
        #[inline]
        fn div(self, rhs: f64) -> f64 {
            self.value() / rhs
        }
    }

    impl Div<Double> for f64 {
        type Output = f64;
        #[inline]
        fn div(self, rhs: Double) -> f64 {
            self / rhs.value()
        }
    }

    impl DivAssign<Double> for f64 {
        #[inline]
        fn div_assign(&mut self, rhs: Double) {
            *self /= rhs.value();
        }
    }

    impl Rem<Self> for Double {
        type Output = f64;
        #[inline]
        fn rem(self, rhs: Self) -> f64 {
            self.value() % rhs.value()
        }
    }

    impl Rem<f64> for Double {
        type Output = f64;
        #[inline]
        fn rem(self, rhs: f64) -> f64 {
            self.value() % rhs
        }
    }

    impl Rem<Double> for f64 {
        type Output = f64;
        #[inline]
        fn rem(self, rhs: Double) -> f64 {
            self % rhs.value()
        }
    }

    impl RemAssign<Double> for f64 {
        #[inline]
        fn rem_assign(&mut self, rhs: Double) {
            *self %= rhs.value();
        }
    }

    impl Neg for Double {
        type Output = f64;
        #[inline]
        fn neg(self) -> f64 {
            -self.value()
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

    impl_ref_ops!(Double, Add, add, Sub, sub, Mul, mul, Div, div, Rem, rem);

    impl_from!(
        f64,
        (Double),
        u8,
        i8,
        u16,
        i16,
        u32,
        i32,
        u64,
        i64,
        u128,
        i128,
        usize,
        isize,
        f32,
        f64
    );
}

thread_local! {
    #[allow(non_upper_case_globals)]
    pub static DoubleType: UnverifiedDartHandle = {
        let zero = UnverifiedDartHandle::new_f64(0.0);
        zero.get_instance_type().unwrap()
    };
}

impl DartType for Double {
    const THIS: &'static LocalKey<UnverifiedDartHandle> = &DoubleType;
}

impl Deref for Double {
    type Target = UnverifiedDartHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl DartHandle for Double {
    fn handle(&self) -> dart_sys::Dart_Handle {
        self.handle.handle()
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        if handle.is_double() {
            Ok(Self {
                handle,
                value: Cell::new(None),
            })
        } else {
            Err(handle)
        }
    }
}
