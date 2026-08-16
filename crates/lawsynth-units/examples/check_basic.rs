use lawsynth_units::{convert, parse_unit, require_compatible};

fn main() {
    let kilometres = parse_unit("km").unwrap();
    let metres = parse_unit("m").unwrap();
    require_compatible(&kilometres, &metres).unwrap();
    println!("2.4 km = {} m", convert(2.4, &kilometres, &metres).unwrap());
}
