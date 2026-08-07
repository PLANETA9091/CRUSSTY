//! Minimal classfile writer for JNI bridge classes.
//!
//! Every bridge class is structurally identical:
//! ```java
//! public class <name> {          // default package or net.minecraft.* etc.
//!     public static native <ret> <method>(<params>);   // xN
//! }
//! ```
//! Java 8 target (major 52). Native methods carry no Code, so the verifier
//! needs no StackMapTable — the classfile is just a constant pool, headers
//! and method descriptors. Access flags 0x0109 = public|static|native
//! (same as the SDK's weaveMark).

use std::collections::HashMap;

/// Build classfile bytes for a bridge class with `methods` (name, JNI sig).
pub fn bridge_class_bytes(name: &str, methods: &[(&str, &str)]) -> Vec<u8> {
    let mut cp: Vec<(u8, Vec<u8>)> = Vec::new();
    let mut idx: HashMap<String, u16> = HashMap::new();

    let this = push_utf8(&mut cp, &mut idx, name);
    let object = push_utf8(&mut cp, &mut idx, "java/lang/Object");
    let this_class = push_class(&mut cp, &mut idx, this);
    let super_class = push_class(&mut cp, &mut idx, object);

    let mut name_ids = Vec::with_capacity(methods.len());
    let mut desc_ids = Vec::with_capacity(methods.len());
    for (m, s) in methods {
        name_ids.push(push_utf8(&mut cp, &mut idx, m));
        desc_ids.push(push_utf8(&mut cp, &mut idx, s));
    }

    let mut c = Vec::new();
    c.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // magic
    c.extend_from_slice(&[0, 0]); // minor
    c.extend_from_slice(&[0, 52]); // major 52 (Java 8)
    c.extend_from_slice(&(cp.len() as u16 + 1).to_be_bytes()); // cp_count
    for (tag, data) in &cp {
        c.push(*tag);
        c.extend_from_slice(data);
    }

    c.extend_from_slice(&[0, 0x21]); // access: public | super
    c.extend_from_slice(&this_class.to_be_bytes());
    c.extend_from_slice(&super_class.to_be_bytes());
    c.extend_from_slice(&[0, 0]); // interfaces_count
    c.extend_from_slice(&[0, 0]); // fields_count

    c.extend_from_slice(&(methods.len() as u16).to_be_bytes());
    for i in 0..methods.len() {
        // public static native, no attributes
        c.extend_from_slice(&[0x01, 0x09]);
        c.extend_from_slice(&name_ids[i].to_be_bytes());
        c.extend_from_slice(&desc_ids[i].to_be_bytes());
        c.extend_from_slice(&[0, 0]); // attributes_count
    }

    c.extend_from_slice(&[0, 0]); // class attributes_count
    c
}

fn push_utf8(cp: &mut Vec<(u8, Vec<u8>)>, idx: &mut HashMap<String, u16>, s: &str) -> u16 {
    if let Some(&i) = idx.get(s) {
        return i;
    }
    let i = (cp.len() + 1) as u16;
    let mut data = Vec::with_capacity(2 + s.len());
    data.extend_from_slice(&(s.len() as u16).to_be_bytes());
    data.extend_from_slice(s.as_bytes());
    cp.push((1, data)); // tag 1 = CONSTANT_Utf8
    idx.insert(s.to_string(), i);
    i
}

fn push_class(cp: &mut Vec<(u8, Vec<u8>)>, idx: &mut HashMap<String, u16>, name_idx: u16) -> u16 {
    let key = format!("C:{name_idx}");
    if let Some(&i) = idx.get(&key) {
        return i;
    }
    let i = (cp.len() + 1) as u16;
    cp.push((7, name_idx.to_be_bytes().to_vec())); // tag 7 = CONSTANT_Class
    idx.insert(key, i);
    i
}
