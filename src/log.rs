/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! trace {
    ($formatter:literal$(, $($args:expr),*)?) => {{
        use ufmt::{uwrite, uwriteln};

        $crate::stdout().with_lock(|w| {
            let r1 = uwrite!(w, "[TRACE] ");
            let _ = r1.and_then(|()| uwriteln!(w, $formatter $(, $($args),*)?));
        })
    }}
}

#[macro_export]
macro_rules! info {
    ($formatter:literal$(, $($args:expr),*)?) => {{
        use ufmt::{uwrite, uwriteln};

        $crate::stdout().with_lock(|w| {
            let r1 = uwrite!(w, "[INFO] ");
            let _ = r1.and_then(|()| uwriteln!(w, $formatter $(, $($args),*)?));
        })
    }}
}

#[macro_export]
macro_rules! debug {
    ($formatter:literal$(, $($args:expr),*)?) => {{
        use ufmt::{uwrite, uwriteln};

        $crate::stdout().with_lock(|w| {
            let r1 = uwrite!(w, "[DEBUG] ");
            let _ = r1.and_then(|()| uwriteln!(w, $formatter $(, $($args),*)?));
        })
    }}
}

#[macro_export]
macro_rules! warn {
    ($formatter:literal$(, $($args:expr),*)?) => {{
        use ufmt::{uwrite, uwriteln};

        $crate::stdout().with_lock(|w| {
            let r1 = uwrite!(w, "[WARN] ");
            let _ = r1.and_then(|()| uwriteln!(w, $formatter $(, $($args),*)?));
        })
    }}
}

#[macro_export]
macro_rules! error {
    ($formatter:literal$(, $($args:expr),*)?) => {{
        use ufmt::{uwrite, uwriteln};

        $crate::stdout().with_lock(|w| {
            let r1 = uwrite!(w, "[ERROR] ");
            let _ = r1.and_then(|()| uwriteln!(w, $formatter $(, $($args),*)?));
        })
    }}
}
