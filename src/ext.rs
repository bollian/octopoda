// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::future::Future;

pub trait Puts<'target, 'message> {
    type Future: Future;

    fn puts(&'target self, message: &'message str) -> Self::Future;
}
