
use pear::*;

fn main() {
    let e = Err::<(), _>(vec![
        Err::<(), _>(pear!({}, "hewwwo worlds")).wrap_err(|| pear!({}, "foaoapoa")).err().unwrap(),
        pear!({}, "goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! goodbye woirlds! "),
    ])
    .wrap_err(|| pear!({
         //"小林劍󠄁" = std::iter::repeat(7).take(30).collect::<Vec<i32>>(),
         //"Ｈｅｌｌｏ" = std::iter::repeat(7).take(30).collect::<Vec<i32>>(),
         //"fooooo\nbar" = std::iter::repeat(7).take(30).collect::<Vec<i32>>(),
    }, "Ｈｅｌｌｏ, ｗｏｒｌｄ!"))
    .wrap_err(|| pear!({}, " 小林劍󠄁 aaaa"))
    .err().unwrap();
    println!("{:#}", e);

 
    let hello = "hello world";
    let error = pear!({
        hello = hello,
        one_fourty_four = 12 * 12,
    }, "I am an error, {:?} is a tuple", (1, 2));

    println!("{}", error);
}