use jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_pawnrace_PawnRace_play<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    colour: JString<'a>,
    output: JObject<'a>,
    input: JObject<'a>,
) {
    // process args
    let colour: String = env
        .get_string(&colour)
        .expect("Couldn't read JNI colour input")
        .into();
    let mut io = IO::new(input, output, env);

    let test = io.recv();
    io.send(test);
}

struct IO<'a> {
    input: JObject<'a>,
    output: JObject<'a>,
    env: JNIEnv<'a>,
}

impl<'a> IO<'a> {
    fn recv(&mut self) -> String {
        // call
        let js: JString = self
            .env
            .call_method(&self.input, "readLine", "()Ljava/lang/String;", &[])
            .unwrap()
            .l()
            .unwrap()
            .into();

        // convert string
        let s: String = self.env.get_string(&js).unwrap().into();

        s
    }

    fn send(&mut self, s: impl AsRef<str>) {
        // convert string
        let js = self.env.new_string(s).unwrap();

        // call
        self.env
            .call_method(
                &self.output,
                "println",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&js)],
            )
            .unwrap();
    }

    fn new(input: JObject<'a>, output: JObject<'a>, env: JNIEnv<'a>) -> IO<'a> {
        Self { input, output, env }
    }
}