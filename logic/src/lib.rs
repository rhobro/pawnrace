use jni::{objects::{JClass, JObject, JString}, sys::{jboolean, jchar, jint}, JNIEnv};

#[no_mangle]
pub extern "system" fn Java_pawnrace_PawnRace_play<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    colour: JString<'local>,
    output: JObject<'local>,
    input: JObject<'local>
) {
    
}
