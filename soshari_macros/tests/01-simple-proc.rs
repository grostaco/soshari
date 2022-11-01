use soshari_macros::generate_adjectives;

generate_adjectives! {
    a, b, c, d
}

fn main() {
    let flags = bitflags_to_adjectives(Adjectives::A | Adjectives::B | Adjectives::C);
    println!("{:#?}", flags);
}
