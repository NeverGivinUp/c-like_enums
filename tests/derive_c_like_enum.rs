#[macro_use]
extern crate c_like_try_from;


use std::convert::TryFrom;

use try_from_int_error::TryFromIntError;

#[derive(Copy, Clone, PartialEq, Eq, Debug, CLikeTryFrom)]
#[repr(u16)]
enum Foo {
    Bar1 = 1u16,
    Bar2 = 2u16,
}


#[test]
fn derive() {
    let bar1 = Foo::Bar1;
    let num = bar1 as u16;
    assert_eq!(1, num);
    let foo_again = Foo::try_from(1u16).unwrap();
    assert_eq!(bar1, foo_again);
    let foo_onece_over = Foo::try_from(num).unwrap();
    assert_eq!(bar1, foo_onece_over);
}

#[test]
fn undefined_err() {
    #[allow(unused_variables)]
        let foo = Foo::try_from(99u16).unwrap_err();
}