use serde::{Deserialize, Serialize};
use soshari_macros::{adjectives, generate_adjectives};

generate_adjectives! {
    a, b, c, d
}

#[derive(Serialize, Deserialize)]
#[adjectives(a, b, c, d)]
pub struct Foo {
    x: Vec<u64>,
    adjectives: Vec<FooAdjectives>,
    id: u64,
}

// impl Foo {
//     fn baz() {}
// }

// fn x<U: PartialEq<u64>>(n: U) -> bool {
//     n == 1u64
// }

// impl FooGroup {
//     fn get<U>(&self, id: U) -> Option<&Foo>
//     where
//         U: Into<u64>,
//     {
//         self.0.iter().find(|n| n.id == id.into())
//     }
// }

fn main() {
    let flags = bitflags_to_adjectives(Adjectives::A | Adjectives::B | Adjectives::C);

    println!("{:#?}", flags);
}
