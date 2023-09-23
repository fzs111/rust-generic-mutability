#![cfg(test)]

use std::ops::{IndexMut, Deref, DerefMut};

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

#[test]
fn map_macro() {
    struct Container{
        inner: InnerContainer,
    }

    struct InnerContainer {
        value: Vec<i32>
    }

    impl Deref for Container{
        type Target = InnerContainer;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl DerefMut for Container {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    let mut c = Container{
        inner: InnerContainer {
            value: vec![1,2,3]
        }
    };

    let gen_c = GenRef::from(&mut c);
    let mut gen_inner = gen_ref!(gen_c -> &gen gen_c.inner);
    gen_inner.as_mut().value.push(4);
    assert_eq!(c.inner.value, vec![1,2,3,4]);

    let gen_c = GenRef::from(&mut c);
    let mut gen_inner = gen_ref!(gen_c -> &gen **gen_c);
    gen_inner.as_mut().value.push(5);
    assert_eq!(c.inner.value, vec![1,2,3,4,5]);

    let gen_c = GenRef::from(&mut c);
    let mut gen_vec = gen_ref!(gen_c -> &gen gen_c.value);
    *gen_vec.as_mut() = vec![10, 11, 12];
    assert_eq!(c.inner.value, vec![10, 11, 12]);

    let gen_c = GenRef::from(&mut c);
    let vec_item = gen_ref!(gen_c -> &gen gen_c.value[1]).into_mut();
    *vec_item = 101;
    assert_eq!(c.inner.value, vec![10, 101, 12]);

}
//TODO: Add MORE TESTS!!!