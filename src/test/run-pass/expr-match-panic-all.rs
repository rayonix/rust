// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.




// When all branches of a match expression result in panic, the entire
// match expression results in panic.
pub fn main() {
    let _x =
        match true {
          true => { 10i }
          false => { match true { true => { panic!() } false => { panic!() } } }
        };
}
