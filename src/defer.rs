/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub struct Defer<F: FnOnce()>(Option<F>);

pub fn defer<F: FnOnce()>(deferred: F) -> Defer<F> {
    Defer(Some(deferred))
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f()
        }
    }
}
