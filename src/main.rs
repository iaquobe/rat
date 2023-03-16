fn main() {
    println!("Hello, world!");

    let mut v = vec![(1,1),(1,2),(2,2),(2,1),(3,1),(1,3)];
    v.sort();
    print!("{:?}", v); 
}
