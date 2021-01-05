use glib::translate::from_glib_full;
use std::ffi::CString;
use webkit2gtk::WebsiteDataManager;
use webkit2gtk_sys::webkit_website_data_manager_new;

// https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebsiteDataManager.html#webkit-website-data-manager-new
pub fn create_website_data_manager(data_dir: &str) -> WebsiteDataManager {
    let data_dir = CString::new(data_dir).unwrap();

    let base_cache_dir = CString::new("base-cache-directory").unwrap();
    let base_data_dir = CString::new("base-data-directory").unwrap();
    let disk_cache_dir = CString::new("disk-cache-directory").unwrap();
    let hsts_cache_dir = CString::new("hsts-cache-directory").unwrap();
    let indexeddb_dir = CString::new("indexeddb-directory").unwrap();
    let local_storage_dir = CString::new("local-storage-directory").unwrap();
    let offline_app_cache_dir = CString::new("offline-application-cache-directory").unwrap();
    let websql_dir = CString::new("websql-directory").unwrap();

    unsafe {
        from_glib_full(webkit_website_data_manager_new(
            base_cache_dir.as_ptr(),
            data_dir.as_ptr(),
            base_data_dir.as_ptr(),
            data_dir.as_ptr(),
            disk_cache_dir.as_ptr(),
            data_dir.as_ptr(),
            hsts_cache_dir.as_ptr(),
            data_dir.as_ptr(),
            indexeddb_dir.as_ptr(),
            data_dir.as_ptr(),
            local_storage_dir.as_ptr(),
            data_dir.as_ptr(),
            offline_app_cache_dir.as_ptr(),
            data_dir.as_ptr(),
            websql_dir.as_ptr(),
            data_dir.as_ptr(),
            std::ptr::null::<i8>(),
        ))
    }
}
