#![macro_use]

macro_rules! element_index {
    ($ElementIndex: ident, $Element: ty) => {
        #[derive(Copy, Clone, Debug)]
        #[derive(PartialEq, Eq)]
        #[derive(PartialOrd, Ord)]
        pub struct $ElementIndex(pub usize);

        impl ElementIndex for $ElementIndex {
            type Element = $Element;
            fn index(self) -> usize { self.0 }
        }
    };
}
