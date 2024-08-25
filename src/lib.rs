mod void_time_collector;

#[no_mangle]
pub extern "C" fn hello_from_rust() {
    hello_from_rust_impl();
}

pub fn hello_from_rust_impl() {
    println!("Hello from Rust!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        hello_from_rust();
    }
}
