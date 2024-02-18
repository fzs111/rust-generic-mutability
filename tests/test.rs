#![cfg(test)]

use generic_mutability::*;
fn gen_index<M: Mutability>(gen_vec: GenRef<'_, M, Vec<i32>>, idx: usize) -> GenRef<'_, M, i32> {
    gen_mut!{M => {
        let ref_vec = from_gen!(gen_vec);
        into_gen!(&gen ref_vec[idx])
    }}
}
fn gen_get<M: Mutability>(gen_vec: GenRef<'_, M, Vec<i32>>, idx: usize) -> Option<GenRef<'_, M, i32>> {
    gen_mut!{M => {
        let ref_vec = from_gen!(gen_vec);
        switch_mut_shared!(<[_]>::get_mut, <[_]>::get)(ref_vec, idx).map(into_gen!())
    }}
}

#[test]
fn map_macro() {
    let mut vec = vec![1, 2, 3];
    let elem = GenRef::gen_to_mut(gen_index(GenRef::from(&mut vec), 1), Mutable::mutability());
    assert_eq!(*elem, 2);
}
#[test]
fn map_macro_with_get() {
    let mut vec = vec![1, 2, 3];
    let elem = gen_get(GenRef::from(&mut vec), 1);
    //assert_eq!(elem, Some(2));
}