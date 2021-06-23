#![macro_use]
pub mod macros {

    #[macro_export]
    macro_rules! impl_from {
    ($t:ty, $($y:path=>$x:path),+) =>{
        $(
            impl From<$x> for $t {
                fn from(args:$x) -> Self {
                    $y(args)
                }
            }
        )+
    }
}

    #[macro_export]
    macro_rules! impl_display {
    ($t:ty,$($y:path=>$x:path),+) => {

        impl Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $y(p)=>p.fmt(f)?,
                    )+
                };
                Ok(())
            }
        }
    };
}

    #[macro_export]
    macro_rules! impl_error {
    ($t:ty,$($y:path=>$x:path),+) => {
        impl Error for $t {
            fn cause(&self) -> Option<&dyn Error> {
                match *self {
                    $(
                        $y(ref p)=>Some(p),
                    )+
                }
            }
        }
    }
}
    #[macro_export]
    macro_rules! enum_error {
    ($t:ident,$($fullerr:path=>$err:ident),+) =>{
        #[derive(Debug)]
        pub enum $t {
            $(
                $err($err),
            )+
        }

        impl_display!($t,$($fullerr=>$err),+);
        impl_error!($t,$($fullerr=>$err),+);
        impl_from!($t,$($fullerr=>$err),+);

    }
}
}