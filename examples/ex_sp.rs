use std::{rc::Rc, sync::Arc, ops::DerefMut};
#[derive(Debug,Clone)]
struct Student {
    name: String,
}

impl Student {
    pub fn new() -> Self {
        Self {
            name: "abc".to_string(),
        }
    }

    pub fn rename(self:&mut Student, name:&str) {
        self.name = String::from(name);
    }
}


fn main() {
    {
        let s = Box::new(Student::new());
        let mut s1 = s.clone();
        s1.name = String::from("xyz");
        println!("{:?} {:?}", s, s1);
    }

    {
        let mut s = Rc::new(Student::new());
        // let sr = &mut s;
        // sr.rename("abc");

        let s1 = Rc::clone(&s);
        
        // Arc::make_mut(&mut s1).rename("xyz");
        
        // Arc::get_mut(&mut s1).unwrap().name = String::from("xyz");
        println!("{:?} {:?}", Rc::get_mut(&mut s), s1);
    }

    {
        let mut s = Arc::new(Student::new());
        // let sr = &mut s;
        // sr.rename("abc");

        let s1 = Arc::clone(&s);
        
        // Arc::make_mut(&mut s1).rename("xyz");
        
        // Arc::get_mut(&mut s1).unwrap().name = String::from("xyz");
        println!("{:?} {:?}", Arc::get_mut(&mut s), s1);
    }
}