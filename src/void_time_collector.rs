use std::ffi::{c_char, CString};
use std::fs;
use std::path::Path;
use url::Url;
use chrono::{DateTime, Local, Utc};
use serde::Deserialize;

#[no_mangle]
// @TODO もしかしたらc_strで受け取るべきかもしれない
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

#[no_mangle]
pub extern "C" fn check_is_in_void_time(ptr: *const VoidTimeCollector, index: usize, target_date: i64) -> bool {
    let collector = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };

    let target_date = DateTime::<Utc>::from_timestamp(target_date, 0).unwrap();
    collector.check_is_in_void_time(index, target_date)
}

#[no_mangle]
pub extern "C" fn get_start_date_by_index(ptr: *const VoidTimeCollector, index: usize, format: *const u8, format_len: usize) -> *mut c_char {
    let collector = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };

    let format = unsafe {
        let slice = std::slice::from_raw_parts(format, format_len);
        std::str::from_utf8(slice).unwrap()
    };

    let start_date = collector.get_start_date_by_index(index, format);
    let start_date = CString::new(start_date).unwrap();
    start_date.into_raw()
}

#[no_mangle]
pub extern "C" fn get_end_date_by_index(ptr: *const VoidTimeCollector, index: usize, format: *const u8, format_len: usize) -> *mut c_char {
    let collector = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };

    let format = unsafe {
        let slice = std::slice::from_raw_parts(format, format_len);
        std::str::from_utf8(slice).unwrap()
    };

    let end_date = collector.get_end_date_by_index(index, format);
    let end_date = CString::new(end_date).unwrap();
    end_date.into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

pub struct VoidTimeCollector {
    dates: Vec<VoidTimeDate>,
}

#[derive(Deserialize)]
struct VoidTimeDate {
    start: DateTime<Local>,
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

    fn check_is_in_void_time(&self, index: usize, target_date: DateTime<Utc>) -> bool {
        let date = &self.dates[index];
        date.start <= target_date && date.end >= target_date
    }

    fn get_start_date_by_index(&self, index: usize, format: &str) -> String {
        self.dates[index].start.format(format).to_string()
    }

    fn get_end_date_by_index(&self, index: usize, format: &str) -> String {
        self.dates[index].end.format(format).to_string()
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

    #[test]
    fn test_check_is_in_void_time() {
        let path = "test_assets/void-time-test.json";
        let collector =
            void_time_collector_new(path.as_ptr(), path.len());

        let date = "2024-01-03T08:36:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let is_in_void_time = check_is_in_void_time(collector, 0, date.timestamp());
        assert_eq!(is_in_void_time, false);

        let date = "2024-01-03T08:37:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let is_in_void_time = check_is_in_void_time(collector, 0, date.timestamp());
        assert_eq!(is_in_void_time, true);

        let date = "2024-01-03T09:00:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let is_in_void_time = check_is_in_void_time(collector, 0, date.timestamp());
        assert_eq!(is_in_void_time, true);

        let date = "2024-01-03T09:48:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let is_in_void_time = check_is_in_void_time(collector, 0, date.timestamp());
        assert_eq!(is_in_void_time, true);

        let date = "2024-01-03T09:49:00+09:00".parse::<DateTime<Utc>>().unwrap();
        let is_in_void_time = check_is_in_void_time(collector, 0, date.timestamp());
        assert_eq!(is_in_void_time, false);
    }

    #[test]
    fn test_get_start_date_by_index() {
        let path = "test_assets/void-time-test.json";
        let collector =
            void_time_collector_new(path.as_ptr(), path.len());

        let format = "%m/%d";
        let start_date_ptr = get_start_date_by_index(collector, 0, format.as_ptr(), format.len());
        let start_date = unsafe {
            let c_str = std::ffi::CStr::from_ptr(start_date_ptr);
            c_str.to_str().unwrap()
        };
        assert_eq!(start_date, "01/03");
        free_string(start_date_ptr);

        let format = "%H:%M ~";
        let start_date_ptr = get_start_date_by_index(collector, 0, format.as_ptr(), format.len());
        let start_date = unsafe {
            let c_str = std::ffi::CStr::from_ptr(start_date_ptr);
            c_str.to_str().unwrap()
        };
        assert_eq!(start_date, "08:37 ~");
        free_string(start_date_ptr);
    }

    #[test]
    fn test_get_end_date_by_index() {
        let path = "test_assets/void-time-test.json";
        let collector =
            void_time_collector_new(path.as_ptr(), path.len());

        let format = "~ %H:%M";
        let end_date_ptr = get_end_date_by_index(collector, 0, format.as_ptr(), format.len());
        let end_date = unsafe {
            let c_str = std::ffi::CStr::from_ptr(end_date_ptr);
            c_str.to_str().unwrap()
        };
        assert_eq!(end_date, "~ 09:48");
        free_string(end_date_ptr);
    }
}
