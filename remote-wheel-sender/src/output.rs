use string_cache::DefaultAtom;

#[derive(Clone, Debug)]
pub enum OutputEvent {
    UpdateAxis(DefaultAtom, f64),
    UpdateButton(DefaultAtom, bool),
    Flush,
}
