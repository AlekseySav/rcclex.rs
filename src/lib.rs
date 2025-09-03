mod charsets;
mod utnfa;

pub use charsets::{Charset, Utf8Charset};
pub use utnfa::UTnfa;

fn _a() {
    let _t = UTnfa::empty();
}
