use std::error::Error;

mod arp;
mod conf;
mod dnspod;
mod mnotify;

// #[derive(Debug)]
// pub enum List<T> {
//     Conis(T, Box<List<T>>),
//     Nil,
// }
//
// impl<T> List<T> {
//     pub fn iter(&self) -> ListIter<T> {
//         return ListIter::<T> { p: self };
//     }
// }
// pub struct ListIter<'a, T> {
//     p: &'a List<T>,
// }
// impl<'a, T> Iterator for ListIter<'a, T>
// where
//     T: Copy,
// {
//     type Item = &'a T;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match &*self.p {
//             List::Conis(c, t) => {
//                 self.p = &*t;
//                 Some(c)
//             }
//             List::Nil => None,
//         }
//     }
// }
//
// pub fn newList<T>(l: &[T]) -> List<T>
// where
//     T: Copy,
// {
//     if l.len() == 0 {
//         return List::Nil;
//     }
//     return List::Conis(l[0], Box::new(newList(&l[1..])));
// }
//
// pub fn newList1<T>(l: &[T]) -> List<T>
// where
//     T: Copy,
// {
//     if l.len() == 0 {
//         return List::Nil;
//     }
//     let mut r = List::Nil;
//     for &x in l.iter().rev() {
//         r = List::Conis(x, Box::new(r))
//     }
//     return r;
// }

fn main() -> Result<(),Box<dyn Error>> {
    let  d = dnspod::DnsPod::new();

    let  f = mnotify::not();
    loop {
        d.handle(f.recv()?)?;
    }
}
