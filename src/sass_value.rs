//! Wrap the Sass Values, see the libsass documentation:
//! https://github.com/sass/libsass/wiki/API-Sass-Value

use sass_sys;
use std::ffi;
use std::fmt;
use util;
use raw::SassValueRaw;

/// Wrap a raw sass value.
pub struct SassValue {
    raw: * mut SassValueRaw,
    is_const: bool
}

impl SassValue {
    // Wrap a read only Sass Value pointer coming from libsass.
    pub fn from_raw(raw: *const SassValueRaw) -> SassValue {
        SassValue {
            raw: raw as *mut SassValueRaw,
            is_const: true
        }
    }

  /// Create a raw SassValueBuf containing a sass string.
    pub fn sass_string(input:&str) -> SassValue {
        let c_str = ffi::CString::new(input).unwrap();
        SassValue {
            raw: unsafe { sass_sys::sass_make_string(c_str.as_ptr()) },
            is_const: false
        }
    }

  /// Create a raw SassValueBuf containing a sass string.
    pub fn sass_error(input:&str) -> SassValue {
        let c_str = ffi::CString::new(input).unwrap();
        SassValue {
            raw: unsafe { sass_sys::sass_make_error(c_str.as_ptr()) },
            is_const: false
        }
    }

    /// return a mutable raw, if available.
    pub fn as_raw(&self) -> Option<* mut SassValueRaw> {
        if self.is_const {
            None
        } else {
            Some(self.raw)
        }
    }


    /// Attempt to extract a String from the raw value.
    pub fn to_string(&self) -> Option<String> {
        if unsafe{sass_sys::sass_value_is_string(self.raw)} != 0 {
            Some(util::build_string(unsafe{sass_sys::sass_string_get_value(self.raw)}))
        } else {
            None
        }
    }

}

impl fmt::Display for SassValue {

    /// Format arbitrary Sass Values
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        fn fmt_list(value: * const SassValueRaw)  -> String {
            let len = unsafe { sass_sys::sass_list_get_length(value) };
            let mut out = String::new();
            out.push_str("[");
            for i in 0..len {
                let entry = unsafe {sass_sys::sass_list_get_value(value,i)};
                if i>0 {
                    out.push_str(", ");
                }
                out.push_str(fmt_raw(entry).as_slice());
            }
            out.push_str("]");
            out
        }

        fn fmt_raw(value: * const SassValueRaw) -> String {
            let sass_tag = unsafe {sass_sys::sass_value_get_tag(value)};
            match sass_tag {
                sass_sys::SASS_LIST =>  fmt_list(value),
                sass_sys::SASS_STRING => util::build_string(
                  unsafe{sass_sys::sass_string_get_value(value)}),
                sass_sys::SASS_BOOLEAN => {
                    let v = unsafe{ sass_sys::sass_boolean_get_value(value) };
                    if v != 0 {
                        String::from_str("true")
                    } else {
                        String::from_str("false")
                    }
                },
                sass_sys::SASS_NUMBER => {
                    let v = unsafe { sass_sys::sass_number_get_value(value)};
                    format!("{}",v)
                },
                sass_sys::SASS_COLOR => {String::from_str("color(?,?,?,?)")},
                sass_sys::SASS_MAP => {String::from_str("{?,?}")},
                sass_sys::SASS_NULL => String::from_str("(null)"),
                sass_sys::SASS_ERROR => util::build_string(
                    unsafe {sass_sys::sass_error_get_message(value)}
                ),
                sass_sys::SASS_WARNING => util::build_string(
                    unsafe {sass_sys::sass_error_get_message(value)}
                ),
                _ => format!("bad sass tag {}", sass_tag)
            }
        }

        fmt.pad_integral(true, "", fmt_raw(self.raw).as_slice())

    }
}


/// An owned SassValueBuf.
pub struct SassValueBuf {
    buf: * mut SassValue,

}


impl SassValueBuf {
    pub fn from_buf(input:&mut SassValue) -> SassValueBuf {
        SassValueBuf {
            buf: input
        }
    }
}


impl Drop for SassValueBuf {
    fn drop(&mut self) {
        unsafe {
            sass_sys::sass_delete_value(self.buf as *mut SassValueRaw)
        }
    }
}
