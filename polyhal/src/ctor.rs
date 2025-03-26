use core::slice::Iter;

extern "Rust" {
    /// The start symbol of the init section
    pub fn __start_ph_init();
    /// The stop symbol of the init section
    pub fn __stop_ph_init();
}

/// PolyHAL's Initialize Wrapper
///
/// This struct contians' constructor function
/// and its priority.
/// The lower the priority, the earlier it will be called
pub struct PHInitWrap {
    /// The priority of the init function
    pub priority: usize,
    /// The Initialize function
    pub func: fn(),
}

/// Polyhal Constructor placeholder
#[used(linker)]
#[unsafe(link_section = "ph_init")]
static PH_INIT_ARR: [PHInitWrap; 0] = [];

/// Get a iterator of the polyhal init section.
///
/// The item of the iterator is function reference.
///
/// ## Demo
///
/// ```rust
/// // Call all initialize function.
/// ph_init_iter().for_each(|f| f());
/// ```
pub fn ph_init_iter<'a>(priority: usize) -> impl Iterator<Item = &'a PHInitWrap> {
    let len = (__stop_ph_init as usize - __start_ph_init as usize) / size_of::<PHInitWrap>();
    unsafe {
        core::slice::from_raw_parts_mut(__start_ph_init as *mut PHInitWrap, len)
            .iter()
            .filter(move |x| x.priority == priority)
    }
}

/// Definiation a constructer
///
/// This constructor will be called by polyhal when booting.
/// Please add `#![feature(used_with_arg)]` at the top of your `lib.rs` file.
///
/// ## Demo
///
/// ```rust
/// ph_ctor!(ctor_name, || {
///     // Ctor block
/// });
/// ```
#[macro_export]
macro_rules! ph_ctor {
    ($name:ident, $f:expr) => {
        #[used(linker)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = "ph_init")]
        static $name: $crate::ctor::PHInitWrap = $crate::ctor::PHInitWrap {
            priority: 0,
            func: $f,
        };
    };
}
