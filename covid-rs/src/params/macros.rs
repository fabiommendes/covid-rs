/// Create methods for EpiLocalParams or EpiParamsData traits
#[macro_export]
macro_rules! epi_param_methods {
    // Create functions that receive no arguments but self
    (
        $(by_field: { $($name:ident),* $(,)? })?
        $(by_value: { $($vname:ident: $value:expr),* $(,)? })?
        $(delegate[$delegate:ident]: { $($dname:ident),* $(,)? })?
        $(forward[$forward:ident]: { $($fname:ident),* $(,)? })?
    ) => {
        $($(
            fn $name(&self) -> Real {
                self.$name
            }
        )*)*
        $($(
            fn $vname(&self) -> Real {
                $value
            }
        )*)*
        $($(
            paste! {
                fn $dname(&self) -> Real {
                    self.$delegate.$dname()
                }
            }
        )*)*
        $($(
            paste! {
                fn $fname(&self) -> Real {
                    self.$forward.$fname
                }
            }
        )*)*
    };

    // Create functions that bind to an argument of type $ty
    (
        $(by_field[$ty:ident]: { $($name:ident),* $(,)? })?
        $(by_value[$vty:ident]: { $($vname:ident: $value:expr),* $(,)? })?
    ) => {
        $($(
            fn $name(&self, obj: &$ty) -> Real {
                self.$name.for_state(obj)
            }
        )*)*
        $($(
            fn $vname(&self, _: &$vty) -> Real {
                $value
            }
        )*)*
    };
}

/// Create a method for EpiLocalParams or SEIRParamsData traits
#[macro_export]
macro_rules! epi_param_method {
    ($name:ident[$ty:ty], delegate=$delegate:ident) => {
        paste! {
            fn $name(&self, obj: &$ty) -> Real {
                self.$delegate.$name(obj)
            }
        }
    };
    ($name:ident<$ty:ident>) => {
        pub fn $name<$ty>(&self, obj: &$ty) -> Real
        where
            T: ForBind<$ty, Output = Real>,
        {
            self.$name.for_state(obj)
        }
    };
    (data = $name:ident[$ty:ty]) => {
        paste! {
            fn [<with_ $name _data>]<S>(&self, f: impl FnOnce(&$ty) -> S) -> S {
                f(&self.$name)
            }
        }
    };
    (data = $name:ident[$ty:ty], value = $value:expr) => {
        paste! {
            fn [<with_ $name _data>]<S>(&self, f: impl FnOnce(&$ty) -> S) -> S
            {
                self.with_scalar_data($value, f)
            }
        }
    };
    (data = $name:ident[$ty:ty], delegate=$delegate:ident) => {
        paste! {
            fn [<with_ $name _data>]<S>(&self, f: impl FnOnce(&$ty) -> S) -> S {
                f(&self.$delegate.$name)
            }
        }
    };
}
