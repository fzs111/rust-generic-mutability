#![cfg(test)]

use generic_mutability::*;

#[test]
fn mut_create_extract(){
    let mut string = String::from("asd");

    let mut maybe_mut = MaybeMut::from(&mut string);

    let mut_ref = maybe_mut.as_mut();

    mut_ref.push('f');

    assert_eq!(string, String::from("asdf"));
}