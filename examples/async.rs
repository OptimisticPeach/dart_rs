#![crate_type = "cdylib"]

use dart::{create_init_function, export_dart_functions};

use dart::dart_cobject::{CObject, TypedDataArray};
use dart::dart_handle::Port;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn random_array(message: CObject, _port: Port) {
    if let CObject::Array(data) = message {
        if let [CObject::Int32(seed), CObject::Int32(length), CObject::SendPort(port)] = &*data {
            let mut rng = StdRng::seed_from_u64(*seed as _);
            let v = (0..*length as usize)
                .map(|_| rng.gen::<u8>())
                .collect::<Vec<u8>>();
            unsafe {
                let port = Port::from_port(port.0.id).unwrap();
                if port.post_cobject(
                    CObject::TypedData(TypedDataArray::create(v).recast()),
                ) {
                    port.post_cobject(CObject::Null);
                }
            }
        } else {
            panic!("Invalid message data!");
        }
    } else {
        panic!("Invalid message data!");
    }
}

export_dart_functions!(async_exports: ["randomArrayServicePort" -> random_array as async]);
create_init_function!(async_example, [async_exports]);
