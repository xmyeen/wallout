use regex;

fn main() {
    let s = regex::RegexSet::new(&[r"abc"]).unwrap();
    for i in s.matches("abc").into_iter() {
        println!("1,2,3,4: {:?}", i);
    }
}