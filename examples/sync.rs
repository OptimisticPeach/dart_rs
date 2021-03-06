#![crate_type = "cdylib"]

use dart::{create_init_function, dart_unwrap, export_dart_functions};

use dart::prelude::*;
use rand::{
    rngs::{OsRng, StdRng},
    Rng, RngCore, SeedableRng,
};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref RNG: Mutex<Option<Box<dyn RngCore + Send + Sync>>> = Mutex::new(None);
}

fn system_rand(arguments: NativeArguments) {
    let mut rng_provider = RNG.lock().unwrap();
    let integer = if let Some(x) = &mut *rng_provider {
        x.gen::<i64>()
    } else {
        let mut rng = Box::new(OsRng) as Box<dyn RngCore + Send + Sync>;
        let num = rng.gen::<i64>();
        let rng = Some(rng);
        *rng_provider = rng;
        num
    };
    arguments.set_return(*Integer::new(integer));
}

fn system_s_rand(arguments: NativeArguments) {
    let seed = dart_unwrap!(arguments.get_i64_arg(0));
    *RNG.lock().unwrap() = Some(Box::new(StdRng::seed_from_u64(seed as u64)));
    arguments.set_return(*Boolean::new(true))
}

export_dart_functions!(sync_exports: ["systemRand" -> system_rand], ["systemSRand" -> system_s_rand]);
create_init_function!(sync_example, [sync_exports]);
