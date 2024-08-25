use std::fs;
use std::path::Path;
use url::Url;

#[no_mangle]
pub extern "C" fn void_time_collector_new(file_path: *const u8, file_path_len: usize) -> *mut VoidTimeCollector {
    let file_path = unsafe {
        let slice = std::slice::from_raw_parts(file_path, file_path_len);
        std::str::from_utf8(slice).unwrap()
    };
    let file_path = Path::new(file_path);
    let collector = Box::new(VoidTimeCollector::new(file_path));
    Box::into_raw(collector)
}

#[no_mangle]
pub extern "C" fn void_time_collector_new_from_url(url: *const u8, url_len: usize) -> *mut VoidTimeCollector {
    let url = unsafe {
        let slice = std::slice::from_raw_parts(url, url_len);
        std::str::from_utf8(slice).unwrap()
    };
    let url = Url::parse(url).unwrap();
    let collector = Box::new(VoidTimeCollector::new_from_url(&url));
    Box::into_raw(collector)
}

#[no_mangle]
pub extern "C" fn void_time_collector_free(ptr: *mut VoidTimeCollector) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

pub struct VoidTimeCollector {
    _dates: Vec<VoidTimeDate>,
}

struct VoidTimeDate {
    _start_time: u64,
    _end_time: u64,
}

impl VoidTimeCollector {
    fn new(file_path: &Path) -> VoidTimeCollector {
        let mut collector = VoidTimeCollector {
            _dates: Vec::new(),
        };
        collector.load_dates(file_path);
        collector
    }

    fn new_from_url(url: &Url) -> VoidTimeCollector {
        println!("new_from_url: {}", url);
        let file_path = url.to_file_path().unwrap();
        VoidTimeCollector::new(&file_path)
    }

    fn load_dates(&mut self, file_path: &Path) {
        println!("load_dates: {}", file_path.display());

        let void_time_string = fs::read_to_string(file_path)
            .expect(&format!("Failed to read file: {}", file_path.display()));
        println!("Void time string: {}", void_time_string);
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    #[test]
    fn test_void_time_collector_new() {
        let path = "test_assets/void-time-test.json";

        let collector =
            void_time_collector_new(path.as_ptr(), path.len());
        void_time_collector_free(collector);
    }

    #[test]
    fn test_void_time_collector_new_from_url() {
        // create URL from current directory
        let url =
            "file://".to_string()
                + env::current_dir().unwrap().to_str().unwrap()
                + "/test_assets/void-time-test.json";
        let url = url.as_str();

        let collector =
            void_time_collector_new_from_url(url.as_ptr(), url.len());
        void_time_collector_free(collector);
    }
}
