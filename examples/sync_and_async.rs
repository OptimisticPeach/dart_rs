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

fn system_s_rand(arguments: NativeArguments) {
    let seed = dart_unwrap!(arguments.get_i64_arg(0));
    *RNG.lock().unwrap() = Some(Box::new(StdRng::seed_from_u64(seed as u64)));
    arguments.set_return(*Boolean::new(true))
}

fn random_array(message: CObject, _port: Port) {
    if let CObject::Array(data) = message {
        if let [CObject::Int32(length), CObject::SendPort(port)] = &*data {
            let mut rng = RNG.lock().unwrap();
            let rng = if let Some(x) = &mut *rng {
                x
            } else {
                *rng = Some(Box::new(OsRng));
                rng.as_mut().unwrap()
            };

            let v = (0..*length as usize)
                .map(|_| rng.gen::<u8>())
                .collect::<Vec<u8>>();
            unsafe {
                let port = Port::from_port(port.0.id).unwrap();
                if port.post_cobject(
                    &mut CObject::TypedData(TypedDataArray::create(v).recast()).into_leak(),
                ) {
                    port.post_cobject(&mut CObject::Null.into_leak());
                }
            }
        } else {
            panic!("Invalid message data!");
        }
    } else {
        panic!("Invalid message data!");
    }
}

export_dart_functions!(exports: ["randomArrayServicePort" -> random_array as async], ["systemSRand" -> system_s_rand]);
create_init_function!(exports_example, [exports]);
