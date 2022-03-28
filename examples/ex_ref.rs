
#[derive(Debug)]
struct Course<'a> {
    student: &'a  mut String
}

#[derive(Debug)]
struct Student<'a> {
    name: String,
    course: Course<'a>,
}

impl<'a> Student<'a> {
    fn new(name: &'a mut String) -> Self {
        Self {
            name: String::from(name.as_str()),
            course: Course {
                student: name
            }
        }
    }
}

fn main() {
    let mut name = (String::from("A"), String::from("B"));
    let mut a = Student::new(&mut name.0);
    let mut b = Student::new(&mut name.1);
    let x = &mut a;
    let y = &mut b;
    
    println!("a:{:?} b:{:?}", x, y);
    {
        x.course.student = &mut x.name;
        // y.course.student = &name.1;
        std::mem::swap(x, y);
    }

    // {
    //     // let x = &mut a;
    //     // let y = &mut b;    
    //     std::mem::swap(&mut a, &mut b);
    // }
    println!("a:{:?} b:{:?}", x, y);
}