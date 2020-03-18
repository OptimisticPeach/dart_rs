use crate::dart_handle::{UnverifiedDartHandle, DartHandle, Error};
use crate::dart_types::d_string::DString;
use crate::dart_unwrap;

#[derive(Copy, Clone)]
pub struct Dynamic {
    handle: UnverifiedDartHandle
}

impl Dynamic {
    pub fn call_function(&self, function: DString, parameters: &mut [UnverifiedDartHandle]) -> Result<Dynamic, Error> {
        self.handle.invoke(function.safe_handle(), parameters).map(Self::from)
    }
    pub fn get_field(&self, field: DString) -> Result<Dynamic, Error> {
        self.handle.get_field(field.safe_handle()).map(Self::from)
    }
    pub fn set_field(&self, field: DString, value: UnverifiedDartHandle) -> Result<(), Error> {
        self.handle.set_field(field.safe_handle(), value)
    }
    pub fn get_property(&self, property: DString) -> Result<Dynamic, Error> {
        self.call_function(property, &mut [])
    }
    pub fn set_property(&self, property: DString, value: UnverifiedDartHandle) -> Result<(), Error> {
        self.call_function(property, &mut [value]).map(drop)
    }
    pub fn get_type(&self) -> Dynamic {
        dart_unwrap!(self.handle.get_instance_type().map(DartHandle::from_handle)).ok().unwrap()
    }
    pub fn type_name(&self) -> String {
        let ty = self.get_type();
        ty.to_string()
    }
    pub fn call_as_function(&self, parameters: &mut [UnverifiedDartHandle]) -> Result<Dynamic, Error> {
        self.handle.invoke_closure(parameters).map(Self::from)
    }
    pub fn from<T: DartHandle>(x: T) -> Self {
        Self {
            handle: x.safe_handle()
        }
    }
}

impl ToString for Dynamic {
    fn to_string(&self) -> String {
        dart_unwrap!(self.handle.to_string()).into_string().unwrap()
    }
}

unsafe impl DartHandle for Dynamic {
    fn handle(&self) -> dart_sys::Dart_Handle {
        self.handle.handle()
    }

    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }

    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        Ok(Self::from(handle))
    }
}

impl std::ops::Deref for Dynamic {
    type Target = UnverifiedDartHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
impl std::ops::DerefMut for Dynamic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

mod impls {
    use super::Dynamic;
    use crate::dart_unwrap;
    use std::ops::{
        Add, AddAssign,
        Sub, SubAssign,
        Mul, MulAssign,
        Div, DivAssign,
        Rem, RemAssign,
        Shl, ShlAssign,
        Shr, ShrAssign,
        BitOr, BitOrAssign,
        BitAnd, BitAndAssign,
        BitXor, BitXorAssign,

        Not, Neg,
    };

    macro_rules! impl_dynamic_ops {
        ($($op:ident, $assign:ident, $op_name:ident, $op_assign_name:ident, $func:ident),*) => {
            $(
                impl $op<Dynamic> for Dynamic {
                    type Output = Dynamic;
                    fn $op_name(self, other: Dynamic) -> Dynamic {
                        Self::from(dart_unwrap!(self.handle.$op_name(*other)))
                    }
                }
                impl<'a> $op<&'a Dynamic> for Dynamic {
                    type Output = Dynamic;
                    fn $op_name(self, other: &'a Dynamic) -> Dynamic {
                        Self::from(dart_unwrap!(self.handle.$op_name(**other)))
                    }
                }
                impl<'a> $op<Dynamic> for &'a Dynamic {
                    type Output = Dynamic;
                    fn $op_name(self, other: Dynamic) -> Dynamic {
                        Dynamic::from(dart_unwrap!(self.handle.$op_name(*other)))
                    }
                }
                impl<'a, 'b> $op<&'a Dynamic> for &'b Dynamic {
                    type Output = Dynamic;
                    fn $op_name(self, other: &'a Dynamic) -> Dynamic {
                        Dynamic::from(dart_unwrap!(self.handle.$op_name(**other)))
                    }
                }

                impl $assign<Dynamic> for Dynamic {
                    fn $op_assign_name(&mut self, other: Dynamic) {
                        *self = self.$op_name(other);
                    }
                }
                impl<'a> $assign<&'a Dynamic> for Dynamic {
                    fn $op_assign_name(&mut self, other: &'a Dynamic) {
                        *self = self.$op_name(other);
                    }
                }
            )*
        };
    }

    impl_dynamic_ops!(
        Add, AddAssign, add, add_assign, op_add,
        Sub, SubAssign, sub, sub_assign, op_sub,
        Mul, MulAssign, mul, mul_assign, op_mul,
        Div, DivAssign, div, div_assign, op_div,
        Rem, RemAssign, rem, rem_assign, op_rem,
        Shl, ShlAssign, shl, shl_assign, op_shl,
        Shr, ShrAssign, shr, shr_assign, op_shr,
        BitOr, BitOrAssign, bitor, bitor_assign, op_bitor,
        BitXor, BitXorAssign, bitxor, bitxor_assign, op_bitxor,
        BitAnd, BitAndAssign, bitand, bitand_assign, op_bitand
    );

    impl Not for Dynamic {
        type Output = Dynamic;
        fn not(self) -> Self::Output {
            Self::from(dart_unwrap!(self.handle.op_bit_not()))
        }
    }

    impl<'a> Not for &'a Dynamic {
        type Output = Dynamic;
        fn not(self) -> Self::Output {
            Dynamic::from(dart_unwrap!(self.handle.op_bit_not()))
        }
    }

    impl Neg for Dynamic {
        type Output = Dynamic;
        fn neg(self) -> Self::Output {
            Self::from(dart_unwrap!(self.handle.op_neg()))
        }
    }

    impl<'a> Neg for &'a Dynamic {
        type Output = Dynamic;
        fn neg(self) -> Self::Output {
            Dynamic::from(dart_unwrap!(self.handle.op_neg()))
        }
    }
}
