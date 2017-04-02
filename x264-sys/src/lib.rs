// TODO do w/out the unions?
// #![feature(untagged_unions)]

pub mod x264;

#[cfg(test)]
mod tests {
    use super::x264::*;
    use std::mem;
    #[test]
    fn init_and_version() {

    }
}
