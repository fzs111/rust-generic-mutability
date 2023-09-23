#![cfg(test)]

use std::ops::IndexMut;

use generic_mutability::*;

#[test]
fn mut_create_extract(){
    let mut string = String::from("asd");

    let mut gen_ref = GenRef::from(&mut string);

    let mut_ref = gen_ref.as_mut();

    mut_ref.push('f');

    assert_eq!(string, String::from("asdf"));
}

fn index_genref<'a, M: Mutability, T, I>(mut self_: GenRef<'a, M, T>, i: I) -> GenRef<'a, M, T::Output>
where
    T: IndexMut<I> + ?Sized
{
    self_.map_with_move(i, |i, t| &mut t[i], |i, t| &t[i])
}

#[test]
fn use_generic_index(){
    let mut v = vec![1, 2, 3, 4];

    let mut gen_v = GenRef::from(&mut v);

    let mut gen_elem = index_genref(gen_v.reborrow(), 2);

    let elem = gen_elem.as_mut();

    assert_eq!(*elem, 3);

    *elem = 13;

    assert_eq!(gen_v.as_immut(), &[1, 2, 13, 4]);

    assert_eq!(v, &[1, 2, 13, 4]);
}

//TODO: Add MORE TESTS!!!