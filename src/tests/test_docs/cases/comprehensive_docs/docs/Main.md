# Main

Defined in comprehensive-docs@0.1.0

## Values

### namespace Main

#### main

Type: `Std::IO ()`

Main entry point

### namespace Main::Inner

#### inner_value

Type: `Std::String`

A value inside namespace

### namespace Main::MyAppendElems

#### my_append_elems

Type: `[a : Std::ToString, c : Main::MyAppendElems, it : Std::Iterator, Std::Iterator::Item it = a] it -> c -> Std::String`

Trait member of `Main::MyAppendElems`

Writes the elements of the iterator after the value

### namespace Main::MyTrait

#### my_method

Type: `[a : Main::MyTrait] a -> Std::String`

Trait member of `Main::MyTrait`

Converts the value to a descriptive string

## Types and aliases

### namespace Main

#### MyIterHolder

Defined as: `type MyIterHolder it = unbox struct { ...fields... }`

A struct holding an iterator

##### field `iter`

Type: `it`

#### MyStruct

Defined as: `type MyStruct = unbox struct { ...fields... }`

A struct with one field

##### field `field`

Type: `Std::I64`

#### MyTypeAlias

Defined as: `type MyTypeAlias = Std::String`

A type alias

#### MyUnion

Defined as: `type MyUnion = unbox union { ...variants... }`

A union with one variant

##### variant `variant`

Type: `Std::I64`

### namespace Main::Inner

#### InnerStruct

Defined as: `type InnerStruct = unbox struct { ...fields... }`

A struct inside namespace

##### field `inner_field`

Type: `Std::I64`

#### InnerTypeAlias

Defined as: `type InnerTypeAlias = Std::String`

A type alias inside namespace

#### InnerUnion

Defined as: `type InnerUnion = unbox union { ...variants... }`

A union inside namespace

##### variant `inner_variant`

Type: `Std::I64`

## Traits and aliases

### namespace Main

#### trait `c : MyAppendElems`

A trait whose member constrains an associated type

##### method `my_append_elems`

Type: `[it : Std::Iterator, a : Std::ToString, Std::Iterator::Item it = a] it -> c -> Std::String`

Writes the elements of the iterator after the value

#### trait `a : MyTrait`

A trait with one member

##### method `my_method`

Type: `a -> Std::String`

Converts the value to a descriptive string

#### trait `MyTraitAlias = Std::ToString`

Kind: `*`

A trait alias

### namespace Main::Inner

#### trait `a : InnerTrait`

A trait inside namespace

#### trait `InnerTraitAlias = Std::ToString`

Kind: `*`

A trait alias inside namespace

## Trait implementations

### impl `Main::Inner::InnerStruct : Main::Inner::InnerTrait`

Implementation of InnerTrait for InnerStruct

### impl `[f : *->*, it : Std::Iterator, Std::Iterator::Item it = f Std::I64] Main::MyIterHolder it : Main::MyTrait`

Implementation of MyTrait for MyIterHolder, whose elements are wrapped in a type constructor

### impl `[it : Std::Iterator, a : Std::ToString, Std::Iterator::Item it = a] Main::MyIterHolder it : Std::ToString`

Implementation of ToString for MyIterHolder, constraining the elements of the iterator

### impl `Main::MyStruct : Main::MyTrait`

Implementation of MyTrait for MyStruct