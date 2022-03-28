// fn f1() {
//     let mut num1 = 5;
//     let mut f1 = move |x: i32| num1 = x + num1;
//     let data1 = f1(2_i32);
//     println!("num1:{:?} data1:{:?}", num1, data1); //num1:5 data1:()
// }

// fn f2() {
//     let mut num1 = 5;
//     let mut f1 = |x: i32| num1 = x + num1;
//     let data1 = f1(2_i32);
//     println!("num1:{:?} data1:{:?}", num1, data1); //num1:5 data1:()
// }

use futures::stream::Stream;
pub fn max_fail<'a, T>(
    stream : impl Stream<Item = Option<T>> +'a , 
    max_consecutive_fails: usize
) -> impl Stream +'a where T : 'a
{
    use futures::stream::StreamExt;
    let mut consecutive_fails = 0;
    stream.take_while(move |x| async {
        if x.is_some(){
            consecutive_fails = 0;
            true
        }
        else{
            consecutive_fails += 1;
            consecutive_fails != max_consecutive_fails
        }
    })
}

pub fn max_fail<'a, T>(
    stream: impl Stream<Item = Option<T>> + 'a,
    max_consecutive_fails: usize,
) -> impl Stream + 'a where T: 'a,
{
    use futures::stream::StreamExt;
    let mut consecutive_fails = 0;
    stream.take_while(move |x| {
        let t = if x.is_some() {
            consecutive_fails = 0;
            true
        } else {
            consecutive_fails += 1;
            consecutive_fails != max_consecutive_fails
        };
        return async move { t };
    })
}

fn main() {
}