fn link_wildmidi() {
    println!("cargo:rustc-flags=-l WildMidi");
}

fn main() {
    link_wildmidi();
}
