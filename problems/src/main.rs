//Premier probleme: Who is the owner?
// fn ret_string() -> String {
//     String::from("  A String object  ")
// }

// fn main() {
//     let original_string = ret_string(); // Store the returned string
//     let s = original_string.trim(); // s borrows from original_string, which owns the string
//     assert_eq!(s, "A String object");
// }

//Second probleme: Write a OOR (owned or ref) type
use std::ops::{Deref, DerefMut};

enum OOR<'a> {
    Owned(String),
    Borrowed(&'a str),
}

impl<'a> Deref for OOR<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            OOR::Owned(s) => s,
            OOR::Borrowed(s) => s,
        }
    }
}

impl<'a> DerefMut for OOR<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            OOR::Owned(s) => s,
            OOR::Borrowed(s) => {
                *self = OOR::Owned(s.to_string());
                if let OOR::Owned(s) = self {
                    s
                } else {
                    unreachable!()
                }
            }
        }
    }
}

//Select between alternatives
fn choose_str<'a>(s1: &'a str, s2: &'a str, select_s1: bool) -> &'a str {
    if select_s1 {
        s1
    } else {
        s2
    }
}

fn main() {
    // Check Deref for both variants of OOR
    let s1 = OOR::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");

    let mut s2 = OOR::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");

    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, OOR::Owned(_)));
    assert!(matches!(s2, OOR::Borrowed(_)));

    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }

    assert!(matches!(s2, OOR::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");
}
