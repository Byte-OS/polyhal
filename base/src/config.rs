pub const KERNEL_OFFSET: usize = get_kernel_offset();
pub const PAGE_SIZE: usize = get_page_size();

macro_rules! compile_env_or_default {
    ($env:literal, $default:literal) => {
        match option_env!($env) {
            Some(s) => s,
            None => $default
        }
    };
}

const fn get_kernel_offset() -> usize {
    let env_v = compile_env_or_default!("POLYHAL_KERNEL_OFFSET", "0");
    match usize::from_str_radix(env_v, 16) {
        Ok(v) => v,
        Err(_) => 0
    }
}

const fn get_page_size() -> usize {
    let env_v = compile_env_or_default!("POLYHAL_PAGE_SIZE", "0x1000");
    match usize::from_str_radix(env_v, 16) {
        Ok(v) => v,
        Err(_) => 0x1000
    }
}