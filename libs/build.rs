use std::{env, path::PathBuf};

fn main() {
    tonic_build::configure()
        .type_attribute("routeguide.Point", "#[derive(Hash)]")
        .compile(&["protobuf/sudoku_matrix.proto"], &["proto"])
        .unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("helloworld_descriptor.bin"))
        .compile(&["protobuf/.proto"], &["proto"])
        .unwrap();

    tonic_build::compile_protos("protobuf/echo/echo.proto").unwrap();

    tonic_build::compile_protos("protobuf/unaryecho/echo.proto").unwrap();

    tonic_build::configure()
        .server_mod_attribute("attrs", "#[cfg(feature = \"server\")]")
        .server_attribute("Echo", "#[derive(PartialEq)]")
        .client_mod_attribute("attrs", "#[cfg(feature = \"client\")]")
        .client_attribute("Echo", "#[derive(PartialEq)]")
        .compile(&["protobuf/attrs/attrs.proto"], &["proto"])
        .unwrap();

    tonic_build::configure()
        .build_server(false)
        .compile(
            &["protobuf/googleapis/google/pubsub/v1/pubsub.proto"],
            &["protobuf/googleapis"],
        )
        .unwrap();

    build_json_codec_service();

    let smallbuff_copy = out_dir.join("smallbuf");
    let _ = std::fs::create_dir(smallbuff_copy.clone()); // This will panic below if the directory failed to create
    tonic_build::configure()
        .out_dir(smallbuff_copy)
        .codec_path("crate::common::SmallBufferCodec")
        .compile(&["protobuf/helloworld/helloworld.proto"], &["proto"])
        .unwrap();
}

// Manually define the json.helloworld.Greeter service which used a custom JsonCodec to use json
// serialization instead of protobuf for sending messages on the wire.
// This will result in generated client and server code which relies on its request, response and
// codec types being defined in a module `crate::common`.
//
// See the client/server examples defined in `src/json-codec` for more information.
fn build_json_codec_service() {
    let greeter_service = tonic_build::manual::Service::builder()
        .name("Greeter")
        .package("json.helloworld")
        .method(
            tonic_build::manual::Method::builder()
                .name("say_hello")
                .route_name("SayHello")
                .input_type("crate::common::HelloRequest")
                .output_type("crate::common::HelloResponse")
                .codec_path("crate::common::JsonCodec")
                .build(),
        )
        .build();

    tonic_build::manual::Builder::new().compile(&[greeter_service]);
}