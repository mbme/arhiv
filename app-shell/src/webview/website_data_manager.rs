use glib::translate::from_glib_full;
use std::ffi::CString;
use webkit2gtk::WebsiteDataManager;
use webkit2gtk_sys::webkit_website_data_manager_new;

// https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebsiteDataManager.html#webkit-website-data-manager-new
pub fn create_website_data_manager(data_dir: &str) -> WebsiteDataManager {
    unsafe {
        from_glib_full(webkit_website_data_manager_new(
            CString::new("base-cache-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("base-data-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("disk-cache-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("hsts-cache-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("indexeddb-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("local-storage-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("offline-application-cache-directory")
                .unwrap()
                .as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("websql-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            std::ptr::null::<i8>(),
        ))
    }
}
