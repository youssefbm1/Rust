// use clap::Parser;


// fn fibo(n: u32) -> Option<u32> {
//     if n == 0 {
//         Some(0)
//     } else if n == 1 {
//         Some(1)
//     } else {
//         let mut a: u32 = 0;
//         let mut b: u32 = 1;
//         for _ in 2..=n {
//             match a.checked_add(b) {
//                 Some(c) => {
//                     a = b;
//                     b = c;
//                 }
//                 None => return None,
//             }
//         }
//         Some(b)
//     }
// }

// fn main() {
//     for i in 0..=42 {
//         if let Some(f) = fibo(i) {
//             println!("fibo({}) = {}", i, f);
//         } else {
//             println!("Fibo({}) as a u32 cannot be represented, exiting loop.", i);
//             break;
//         }
//     }
// }
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    name = "fibo",
    about = "Compute Fibonacci suite values",
    version = "1.0",
    author = "Your Name"
)]
struct Opts {
    /// The maximal number to print the fibo value of
    value: u32,

    /// Print intermediate values
    #[clap(short, long)]
    verbose: bool,

    /// The minimum number to compute
    #[clap(short, long, default_value = "0")]
    min: u32,
}

fn fibo(n: u32) -> Option<u32> {
    if n == 0 {
        Some(0)
    } else if n == 1 {
        Some(1)
    } else {
        let mut a: u32 = 0;
        let mut b: u32 = 1;
        for _ in 2..=n {
            match a.checked_add(b) {
                Some(c) => {
                    a = b;
                    b = c;
                }
                None => return None,
            }
        }
        Some(b)
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    for i in opts.min..=opts.value {
        if let Some(f) = fibo(i) {
            println!("fibo({}) = {}", i, f);
        } else {
            println!("Fibo({}) as a u32 cannot be represented, exiting loop.", i);
            break;
        }
    }
}


