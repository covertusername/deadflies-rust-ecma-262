use ecma_262::types::Number;

fn main() {
    let num = 943.646;
    let mut rad = 2;

    while rad <= 36 {
        println!("{}, rad {}", num.to_istring(rad), rad);
        rad += 1;
    }
}