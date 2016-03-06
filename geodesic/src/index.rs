use std::ops::{Index, IndexMut};

pub trait ElementIndex {
    type Element;

    fn index(self) -> usize;
}

pub fn get<I: ElementIndex, Elements>(elements: &Elements, index: I) -> &I::Element where
    Elements: Index<usize, Output = I::Element>,
{
    &elements[index.index()]
}

pub fn get_mut<I: ElementIndex, Elements>(elements: &mut Elements, index: I) -> &mut I::Element where
    Elements: IndexMut<usize, Output = I::Element>,
{
    &mut elements[index.index()]
}
