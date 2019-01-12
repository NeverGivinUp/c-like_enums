# C-Like TryFrom

This crate implements the `TryFrom` trait for C-like `enums` of some kind of integer.

For example:

```rust
#[derive(Clone, Debug, PartialEq, Eq, CLikeTryFrom)]
#[repr(i16)] 
pub enum SomeEnum {
    VariantA = -101i16,
    VariantB = 0i16,
    VariantC = 33i16,
}

```

And then you can do:

```rust
#[test]
fn demonstrate() {
    // You can cast the c-like enum as the type of integer it has
    let variant_as_integer:i16 = SomeEnum::VariantA as i16;  
    
    
    // You can convert any integer to the corresponding enum variant
    let enum_variant:SomeEnum = SomeEnum::try_from(variant_as_integer).unwrap();
    assert_eq(SomeEnum::VariantA, enum_variant);  
    
    // The From implementation will return an `Err` on an attempt to convert a undefined integer
    let undefined_enum = SomeEnum::try_from(99i16);
    assert_eq(Err(), undefined_enum);
}
```
