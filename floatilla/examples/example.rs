use floatilla::{*, try_math::{self, TryMath}};
use std::{
    collections::HashMap,
    f64::consts,
};

fn main() {
    let mut hashmap: HashMap<FpRepr<f64>, Vec<i32>> = HashMap::new();
    for n in 0..30 {
        let k = Real::new(n as f64) % r64(6.0) / r64(consts::PI);
        hashmap.entry(k.into()).or_insert_with(Vec::new).push(n);
    }
    for i in 20..24 {
        hashmap.entry(FpRepr::new(f64::INFINITY)).or_insert_with(Vec::new).push(i);
    }
    for i in 30..34 {
        hashmap.entry(FpRepr::new(f64::NEG_INFINITY)).or_insert_with(Vec::new).push(i);
    }
    for i in 40..44 {
        hashmap.entry(FpRepr::new(f64::NAN)).or_insert_with(Vec::new).push(i);
    }

    println!("{:#?}", hashmap);

    let mut keys = hashmap.keys().copied().collect::<Vec<_>>();
    keys.sort();
    println!("{:?}", keys);

    match try_math() {
        Ok(r) => println!("{}", r),
        Err(e) => println!("{}", e),
    }
}

fn try_math() -> try_math::Result<f32> {
    let n = ((TryMath(r32(3.0)) * TryMath(r32(0.5)))? / (TryMath(r32(1.0)) - TryMath(r32(1.0)))?)?;

    Ok(n)
}