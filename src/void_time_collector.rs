use std::fs;
use std::path::Path;
use url::Url;
use chrono::{DateTime, Local, Utc};
use serde::Deserialize;

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

#[no_mangle]
pub extern "C" fn get_void_time_index(ptr: *const VoidTimeCollector, date: i64) -> usize {
    let collector = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };

    let date_time = DateTime::<Utc>::from_timestamp(date, 0).unwrap();
    collector.get_void_time_index(date_time)
}

pub struct VoidTimeCollector {
    dates: Vec<VoidTimeDate>,
}

#[derive(Deserialize)]
struct VoidTimeDate {
    start: DateTime<Utc>,
    end: DateTime<Local>,
}

impl VoidTimeCollector {
    fn new(file_path: &Path) -> VoidTimeCollector {
        let mut collector = VoidTimeCollector {
            dates: Vec::new(),
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

        self.dates = serde_json::from_str(&void_time_string)
            .expect("Failed to parse JSON");
    }

    fn get_void_time_index(&self, date: DateTime<Utc>) -> usize {
        self.dates.iter().position(|d| date <= d.end).unwrap_or(0)
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

    #[test]
    fn test_get_void_time_index() {
        let path = "test_assets/void-time-test.json";
        let collector =
            void_time_collector_new(path.as_ptr(), path.len());

        let date = "2024-01-01T10:00:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let index = get_void_time_index(collector, date.timestamp());
        assert_eq!(index, 0);

        let date = "2024-01-03T09:00:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let index = get_void_time_index(collector, date.timestamp());
        assert_eq!(index, 0);

        let date = "2024-01-03T09:48:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let index = get_void_time_index(collector, date.timestamp());
        assert_eq!(index, 0);

        let date = "2024-01-03T09:49:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let index = get_void_time_index(collector, date.timestamp());
        assert_eq!(index, 1);

        let date = "2024-02-04T15:29:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let index = get_void_time_index(collector, date.timestamp());
        assert_eq!(index, 5);

        let date = "2024-02-04T15:30:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let index = get_void_time_index(collector, date.timestamp());
        assert_eq!(index, 0);

        void_time_collector_free(collector);
    }
}
