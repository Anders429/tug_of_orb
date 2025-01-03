#[derive(Debug)]
#[repr(C, align(4))]
pub struct Align4<T>(pub T);

#[macro_export]
macro_rules! include_bytes_aligned {
    ($file:expr $(,)?) => {{
        crate::align::Align4(*include_bytes!($file))
    }};
}
