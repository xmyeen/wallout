extern crate rcgen;
use rcgen::generate_simple_self_signed;

fn main() {
    // let subject_alt_names = vec!["hello.world.example".to_string(),
	// "localhost".to_string()];

    // let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    // // The certificate is now valid for localhost and the domain "hello.world.example"
    // println!("{}", cert.serialize_pem().unwrap());
    // println!("{}", cert.serialize_private_key_pem());
    let mut vec : Vec<Option<i32>> = Vec::new();
    vec.push(None);
    vec.push(None);
    println!("{}", vec.into_iter().filter_map(|o| o).any(|i| i != 0i32));
}