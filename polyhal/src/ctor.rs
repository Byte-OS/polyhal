extern "Rust" {
    /// The start symbol of the init section
    pub fn __start_ph_init();
    /// The stop symbol of the init section
    pub fn __stop_ph_init();
}

/// Contructor Types
///
/// ## Executeion Flow:
///
/// ```plain
/// Primary Core:
/// Cpu  ->  Platform -> HALDriver -> (optional) Boot Other Core -> KernelService -> Normal -> jump to kernel
///
/// Secondary Core:
/// Cpu  ->  Waiting For Primary Core -> jump to kernel
/// ```
///
/// ## Note
///
/// If your kernel requires a specialized function, using [CtorType::Others] to manage it manually.
#[repr(u8)]
#[derive(PartialEq)]
pub enum CtorType {
    /// Init function for the primary CPU, executed only once.
    Primary,
    /// CPU-related constructor, runs on all CPUs.    
    Cpu,
    /// Platform-level constructor, executed only once on the primary CPU.
    Platform,
    /// HAL driver constructor (e.g., interrupt controller), executed only once on the primary CPU.
    HALDriver,
    /// Kernel service constructor, executed only once on the primary CPU.
    KernelService,
    /// General-purpose constructor, executed only once on the primary CPU.
    Normal,
    /// Custom constructor, not executed by HAL.
    Others(u8),
}

/// PolyHAL's Initialize Wrapper
///
/// This struct contians' constructor function
/// and its priority.
/// The lower the priority, the earlier it will be called
pub struct PHInitWrap {
    /// The priority of the init function
    pub priority: CtorType,
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
pub fn ph_init_iter<'a>(priority: CtorType) -> impl Iterator<Item = &'a PHInitWrap> {
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
    ($name:ident, $ty: expr, $f:expr) => {
        #[used(linker)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = "ph_init")]
        static $name: $crate::ctor::PHInitWrap = $crate::ctor::PHInitWrap {
            priority: $ty,
            func: $f,
        };
    };
}
