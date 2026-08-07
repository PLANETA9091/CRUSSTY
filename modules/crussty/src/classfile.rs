//! Minimal classfile surgery for the `area_map` byte hook.
//!
//! Replaces the body of `SingleUserAreaMap.update(int,int,int)` with a
//! branch-minimal body that delegates the whole difference-apply to the
//! injected `SingleUserAreaMapOps` helper (native enumeration + Java apply
//! loop). The original contract is preserved exactly:
//!   - negative newDistance -> IllegalArgumentException + athrow
//!   - lastChunkX == NOT_SET (i32::MIN) -> return false, fields untouched
//!   - otherwise: write fields, run native ops, return true.
//!
//! All new constant-pool entries are APPENDED (existing indices stay valid).
//! The new method body has branch targets, so its Code attribute carries a
//! StackMapTable (mandatory for class-file major >= 51). One frame per basic
//! block leader (offset 0, the ifge fall-through at 4, the ifge target at 16,
//! the if_icmpne fall-through at 42, the if_icmpne target at 44):
//!   0, 4, 16, 42        -> this, toX, toZ, newD
//!   44                  -> this, toX, toZ, newD, fromX, fromZ, oldD
//!
//! Layout of the new body (locals: 0=this, 1=toX, 2=toZ, 3=newD, 4=fromX,
//! 5=fromZ, 6=oldD, 7=param; max_stack 8 = the static run call). JVM "_n"
//! load/store forms exist ONLY for slots 0-3 (iload=0x15, istore=0x36,
//! aload=0x19, astore=0x3a are the two-byte generic forms), and branch
//! offsets are measured from the address of the opcode, not the next
//! instruction:
//! ```text
//!   0: iload_3
//!   1: ifge +15            -> 16
//!   4: new 0xIllegalArgumentException
//!   7: dup
//!   8: iload_3
//!   9: invokestatic Integer.toString(I)Ljava/lang/String;
//!  12: invokespecial 0xIllegalArgumentException.<init>(Ljava/lang/String;)V
//!  15: athrow
//!  16: aload_0; getfield lastChunkX; istore 4
//!  22: aload_0; getfield lastChunkZ; istore 5
//!  28: aload_0; getfield distance;  istore 6
//!  34: iload 4; ldc_w MIN_VALUE; if_icmpne +5 -> 44
//!  42: iconst_0; ireturn
//!  44: aload_0; iload_1; putfield lastChunkX
//!  49: aload_0; iload_2; putfield lastChunkZ
//!  54: aload_0; iload_3; putfield distance
//!  59: aload_0; getfield parameter; astore 7
//!  65: aload_0; iload 4; iload 5; iload 6; iload_1; iload_2; iload_3; aload 7
//!  77: invokestatic SingleUserAreaMapOps.run(LSingleUserAreaMap;IIIIIILjava/lang/Object;)V
//!  80: iconst_1; ireturn
//! ```

const TAG_UTF8: u8 = 1;
const TAG_INTEGER: u8 = 3;
const TAG_CLASS: u8 = 7;
const TAG_FIELDREF: u8 = 9;
const TAG_METHODREF: u8 = 10;
const TAG_NAMEANDTYPE: u8 = 12;

pub const OPS_CLASS: &str = "ca/spottedleaf/moonrise/common/misc/SingleUserAreaMapOps";
pub const MAP_CLASS: &str = "ca/spottedleaf/moonrise/common/misc/SingleUserAreaMap";

const NIE: &str = "java/lang/IllegalArgumentException";
const INTEGER: &str = "java/lang/Integer";

/// One parsed constant-pool entry: (apparent index, tag, payload).
type Entry = (u16, u8, Vec<u8>);

struct Pool {
    entries: Vec<Entry>,
    /// 1-based index the next appended entry will get; after a parse this
    /// equals cp_count (index space = 1 + sum of apparent slots).
    next: u16,
}

impl Pool {
    /// Parse the constant pool of `bytes` starting at `cp_start`; returns the
    /// pool and the offset just past it (where access_flags begins).
    fn parse(bytes: &[u8], cp_start: usize, cp_count: u16) -> Option<(Pool, usize)> {
        let mut pool = Pool {
            entries: Vec::with_capacity(cp_count as usize),
            next: 1,
        };
        let mut p = cp_start;
        let mut seen = 0u32;
        // JVMS: constant_pool_count = number of entries + 1 (index 0 is
        // reserved); the pool therefore holds cp_count - 1 entries.
        let entries_total = u32::from(cp_count.saturating_sub(1));
        while seen < entries_total {
            let tag = *bytes.get(p)?;
            p += 1;
            let (payload, slots): (Vec<u8>, u16) = match tag {
                TAG_UTF8 => {
                    if p + 2 > bytes.len() {
                        return None;
                    }
                    let len = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
                    // store the whole entry payload (length prefix + bytes) so
                    // utf8()/utf8_value() are symmetric with the file layout
                    let d = bytes.get(p..p + 2 + len)?.to_vec();
                    p += 2 + len;
                    (d, 1)
                }
                TAG_INTEGER | 4 => {
                    let d = bytes.get(p..p + 4)?.to_vec();
                    p += 4;
                    (d, 1)
                }
                5 | 6 => {
                    let d = bytes.get(p..p + 8)?.to_vec();
                    p += 8;
                    (d, 2)
                }
                7 | 8 | 16 | 19 | 20 => {
                    let d = bytes.get(p..p + 2)?.to_vec();
                    p += 2;
                    (d, 1)
                }
                9 | 10 | 11 | 12 | 17 | 18 => {
                    let d = bytes.get(p..p + 4)?.to_vec();
                    p += 4;
                    (d, 1)
                }
                15 => {
                    let d = bytes.get(p..p + 3)?.to_vec();
                    p += 3;
                    (d, 1)
                }
                _ => return None,
            };
            pool.entries.push((pool.next, tag, payload));
            pool.next = pool.next.checked_add(slots)?;
            seen += 1;
        }
        Some((pool, p))
    }

    fn find(&self, tag: u8, payload: &[u8]) -> Option<u16> {
        self.entries
            .iter()
            .find(|(_, t, d)| *t == tag && d == payload)
            .map(|(i, _, _)| *i)
    }

    fn push(&mut self, tag: u8, payload: Vec<u8>, slots: u16) -> u16 {
        let idx = self.next;
        self.entries.push((idx, tag, payload));
        self.next += slots;
        idx
    }

    fn utf8(&mut self, s: &str) -> u16 {
        let mut payload = Vec::with_capacity(2 + s.len());
        payload.extend_from_slice(&(s.len() as u16).to_be_bytes());
        payload.extend_from_slice(s.as_bytes());
        self.find(TAG_UTF8, &payload)
            .unwrap_or_else(|| self.push(TAG_UTF8, payload, 1))
    }

    fn int_const(&mut self, v: i32) -> u16 {
        let payload = v.to_be_bytes().to_vec();
        self.find(TAG_INTEGER, &payload)
            .unwrap_or_else(|| self.push(TAG_INTEGER, payload, 1))
    }

    fn class_of(&mut self, utf8_idx: u16) -> u16 {
        let payload = utf8_idx.to_be_bytes().to_vec();
        self.find(TAG_CLASS, &payload)
            .unwrap_or_else(|| self.push(TAG_CLASS, payload, 1))
    }

    fn name_and_type(&mut self, name: &str, desc: &str) -> u16 {
        let n = self.utf8(name);
        let d = self.utf8(desc);
        let mut payload = n.to_be_bytes().to_vec();
        payload.extend_from_slice(&d.to_be_bytes());
        self.find(TAG_NAMEANDTYPE, &payload)
            .unwrap_or_else(|| self.push(TAG_NAMEANDTYPE, payload, 1))
    }

    fn field_ref(&mut self, owner: &str, name: &str, desc: &str) -> u16 {
        let owner_utf8 = self.utf8(owner);
        let c = self.class_of(owner_utf8);
        let nat = self.name_and_type(name, desc);
        let mut payload = c.to_be_bytes().to_vec();
        payload.extend_from_slice(&nat.to_be_bytes());
        self.find(TAG_FIELDREF, &payload)
            .unwrap_or_else(|| self.push(TAG_FIELDREF, payload, 1))
    }

    fn method_ref(&mut self, owner: &str, name: &str, desc: &str) -> u16 {
        let owner_utf8 = self.utf8(owner);
        let c = self.class_of(owner_utf8);
        let nat = self.name_and_type(name, desc);
        let mut payload = c.to_be_bytes().to_vec();
        payload.extend_from_slice(&nat.to_be_bytes());
        self.find(TAG_METHODREF, &payload)
            .unwrap_or_else(|| self.push(TAG_METHODREF, payload, 1))
    }

    fn utf8_value(&self, idx: u16) -> Option<String> {
        let (_, tag, payload) = self.entries.iter().find(|(i, _, _)| *i == idx)?;
        if *tag != TAG_UTF8 {
            return None;
        }
        let len = usize::from(u16::from_be_bytes([payload[0], payload[1]]));
        String::from_utf8(payload[2..2 + len].to_vec()).ok()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (_, tag, payload) in &self.entries {
            out.push(*tag);
            out.extend_from_slice(payload);
        }
        out
    }
}

struct Method {
    /// Offset of the method's access_flags within the whole classfile.
    start: usize,
    /// Offset just past the method's last attribute.
    end: usize,
    name_idx: u16,
    desc_idx: u16,
    access: u16,
}

/// Locate the method with the given (name_idx, desc_idx) in the method table
/// beginning at `methods_start` (offset of the methods_count field).
fn find_method(
    bytes: &[u8],
    methods_start: usize,
    name_idx: u16,
    desc_idx: u16,
) -> Option<Method> {
    let mut p = methods_start;
    let count = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
    p += 2;
    for _ in 0..count {
        let start = p;
        let access = u16::from_be_bytes([bytes[p], bytes[p + 1]]);
        let n = u16::from_be_bytes([bytes[p + 2], bytes[p + 3]]);
        let d = u16::from_be_bytes([bytes[p + 4], bytes[p + 5]]);
        p += 6;
        let attr_count = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
        p += 2;
        for _ in 0..attr_count {
            let len = u32::from_be_bytes([bytes[p + 2], bytes[p + 3], bytes[p + 4], bytes[p + 5]]);
            p += 6 + len as usize;
        }
        if n == name_idx && d == desc_idx {
            return Some(Method {
                start,
                end: p,
                name_idx: n,
                desc_idx: d,
                access,
            });
        }
    }
    None
}

struct ClassLayout {
    pool: Pool,
    /// Offset of access_flags (end of the constant pool region).
    cp_end: usize,
    /// cpool index of this_class (a CONSTANT_Class entry).
    this_class_idx: u16,
    /// Offset of the methods_count field (start of the method table).
    methods_start: usize,
}

fn parse_layout(bytes: &[u8]) -> Option<ClassLayout> {
    let cp_count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let (pool, cp_end) = Pool::parse(bytes, 10, cp_count)?;
    let this_class_idx = u16::from_be_bytes([bytes[cp_end + 2], bytes[cp_end + 3]]);
    let mut p = cp_end + 6; // access_flags(2) this_class(2) super_class(2)
    let iface_count = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
    p += 2 + 2 * iface_count;
    let fields_count = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
    p += 2;
    for _ in 0..fields_count {
        p += 6;
        let attr_count = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
        p += 2;
        for _ in 0..attr_count {
            let len = u32::from_be_bytes([bytes[p + 2], bytes[p + 3], bytes[p + 4], bytes[p + 5]]);
            p += 6 + len as usize;
        }
    }
    Some(ClassLayout {
        pool,
        cp_end,
        this_class_idx,
        methods_start: p,
    })
}

/// Resolve the internal name of this_class (e.g. "a/b/C") from the pool.
fn this_class_name(layout: &ClassLayout) -> Option<String> {
    let (_, tag, payload) = layout
        .pool
        .entries
        .iter()
        .find(|(i, _, _)| *i == layout.this_class_idx)?;
    if *tag != TAG_CLASS {
        return None;
    }
    let utf8_idx = u16::from_be_bytes([payload[0], payload[1]]);
    layout.pool.utf8_value(utf8_idx)
}

/// Build the patched class bytes for `SingleUserAreaMap`: the constant pool
/// keeps every original entry (append-only), the `update` method's Code is
/// replaced wholesale.
pub fn patch_update(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let layout = parse_layout(bytes).ok_or("bad classfile layout")?;
    let this_name = this_class_name(&layout).ok_or("cannot resolve this_class name")?;
    if this_name != MAP_CLASS {
        return Err(format!("unexpected class {this_name}"));
    }
    let mut pool = layout.pool;

    let update_name = pool.utf8("update");
    let update_desc = pool.utf8("(III)Z");
    let m = find_method(bytes, layout.methods_start, update_name, update_desc)
        .ok_or("update(III)Z not found")?;

    // ---- constant refs needed by the new body (appended when absent) ----
    let f_last_x = pool.field_ref(&this_name, "lastChunkX", "I");
    let f_last_z = pool.field_ref(&this_name, "lastChunkZ", "I");
    let f_dist = pool.field_ref(&this_name, "distance", "I");
    let f_param = pool.field_ref(&this_name, "parameter", "Ljava/lang/Object;");
    let nie_utf8 = pool.utf8(NIE);
    let cls_nie = pool.class_of(nie_utf8);
    let m_nie_init = pool.method_ref(NIE, "<init>", "(Ljava/lang/String;)V");
    let m_to_str = pool.method_ref(INTEGER, "toString", "(I)Ljava/lang/String;");
    let min_int = pool.int_const(i32::MIN);
    let run_desc = format!("(L{this_name};IIIIIILjava/lang/Object;)V");
    let m_run = pool.method_ref(OPS_CLASS, "run", &run_desc);

    let mut code = Vec::with_capacity(82);
    let u2 = |out: &mut Vec<u8>, v: u16| out.extend_from_slice(&v.to_be_bytes());
    // A: throw guard (16 bytes; ifge +15 -> 0x10, offset measured from the
    // opcode address per JVMS, not from the next instruction)
    code.push(0x1d); // iload_3
    code.extend_from_slice(&[0x9c, 0x00, 0x0f]); // ifge +15 -> 0x10
    code.push(0xbb); // new
    u2(&mut code, cls_nie);
    code.push(0x59); // dup
    code.push(0x1d); // iload_3
    code.push(0xb8); // invokestatic
    u2(&mut code, m_to_str);
    code.push(0xb7); // invokespecial
    u2(&mut code, m_nie_init);
    code.push(0xbf); // athrow
    debug_assert_eq!(code.len(), 16);
    // B: snapshot old state (18 bytes). Slots 4-6 need the generic forms:
    // there are NO istore_4.. istore_7 short opcodes (istore<i> = 0x36 + u1).
    code.push(0x2a); // aload_0
    code.push(0xb4); // getfield
    u2(&mut code, f_last_x);
    code.extend_from_slice(&[0x36, 0x04]); // istore 4
    code.push(0x2a);
    code.push(0xb4);
    u2(&mut code, f_last_z);
    code.extend_from_slice(&[0x36, 0x05]); // istore 5
    code.push(0x2a);
    code.push(0xb4);
    u2(&mut code, f_dist);
    code.extend_from_slice(&[0x36, 0x06]); // istore 6
    // C: NOT_SET check. 0xa0 = if_icmpne (0xa4 is if_icmple); there is no
    // iload_4.._6 either, so use generic iload = 0x15 + u1.
    code.extend_from_slice(&[0x15, 0x04]); // iload 4
    code.push(0x13); // ldc_w
    u2(&mut code, min_int);
    code.extend_from_slice(&[0xa0, 0x00, 0x05]); // if_icmpne +5 -> 0x2c
    code.push(0x03); // iconst_0
    code.push(0xac); // ireturn
    debug_assert_eq!(code.len(), 44);
    // D: write fields, run ops
    code.push(0x2a); // aload_0
    code.push(0x1b); // iload_1
    code.push(0xb5); // putfield
    u2(&mut code, f_last_x);
    code.push(0x2a);
    code.push(0x1c); // iload_2
    code.push(0xb5);
    u2(&mut code, f_last_z);
    code.push(0x2a);
    code.push(0x1d); // iload_3
    code.push(0xb5);
    u2(&mut code, f_dist);
    code.push(0x2a);
    code.push(0xb4); // getfield parameter
    u2(&mut code, f_param);
    code.extend_from_slice(&[0x3a, 0x07]); // astore 7
    code.push(0x2a); // aload_0
    code.extend_from_slice(&[0x15, 0x04]); // iload 4
    code.extend_from_slice(&[0x15, 0x05]); // iload 5
    code.extend_from_slice(&[0x15, 0x06]); // iload 6
    code.push(0x1b); // iload_1
    code.push(0x1c); // iload_2
    code.push(0x1d); // iload_3
    code.extend_from_slice(&[0x19, 0x07]); // aload 7
    code.push(0xb8); // invokestatic
    u2(&mut code, m_run);
    code.push(0x04); // iconst_1
    code.push(0xac); // ireturn
    debug_assert_eq!(code.len(), 82, "emitted code is {}", code.len());

    // ---- StackMapTable: one frame per branch target, mirroring javac ----
    // Only the two branch targets (16 and 44) are basic-block leaders that
    // need stack-map entries; full_frame entries at non-leader offsets are
    // rejected by the HotSpot verifier. Frame @16 has the method's initial
    // locals (this + toX + toZ + newD) -> same_frame (16). Frame @44 has
    // those plus the three snapshot int locals (fromX/fromZ/oldD) appended
    // via append_frame (254). Per JVMS 4.10.1.2 each offset_delta after the
    // first frame is offset_of(previous) + offset_delta + 1, so the second
    // entry needs offset_delta 44-16-1 = 27 (0x1b), not 28.
    let mut stackmap = (2u16).to_be_bytes().to_vec();
    stackmap.push(0x10); // same_frame @16 -> absolute frame @16 (initial 4 locals)
    stackmap.extend_from_slice(&[0xfe, 0x00, 0x1b, 0x01, 0x01, 0x01]); // append -> absolute @44

    // ---- Code attribute ----
    let mut code_attr = Vec::new();
    u2(&mut code_attr, pool.utf8("Code"));
    // attribute_length comes after we know the body size
    let mut body = Vec::new();
    u2(&mut body, 8); // max_stack
    u2(&mut body, 8); // max_locals
    body.extend_from_slice(&(code.len() as u32).to_be_bytes());
    body.extend_from_slice(&code);
    body.extend_from_slice(&[0, 0]); // exception_table_length
    body.extend_from_slice(&(1u16).to_be_bytes()); // attributes_count
    u2(&mut body, pool.utf8("StackMapTable"));
    body.extend_from_slice(&(stackmap.len() as u32).to_be_bytes());
    body.extend_from_slice(&stackmap);
    code_attr.extend_from_slice(&(body.len() as u32).to_be_bytes());
    code_attr.extend_from_slice(&body);

    // ---- replacement method entry ----
    let mut method = Vec::new();
    u2(&mut method, m.access);
    u2(&mut method, m.name_idx);
    u2(&mut method, m.desc_idx);
    u2(&mut method, 1); // attributes_count
    method.extend_from_slice(&code_attr);

    // ---- splice: header + new cp + tail with the update method replaced ----
    let mut out = Vec::with_capacity(bytes.len() + 256);
    out.extend_from_slice(&bytes[0..8]); // magic, minor, major
    u2(&mut out, pool.next); // new cp_count
    out.extend_from_slice(&pool.serialize());
    out.extend_from_slice(&bytes[layout.cp_end..m.start]);
    out.extend_from_slice(&method);
    out.extend_from_slice(&bytes[m.end..]);
    Ok(out)
}

#[test]
fn dbg_parse() {
    let bytes = include_bytes!("../tests/fixtures/SingleUserAreaMap.class");
    let cp_count = u16::from_be_bytes([bytes[8], bytes[9]]);
    eprintln!("file len {} cp_count {cp_count}", bytes.len());
    let (pool, end) = crate::classfile::Pool::parse(bytes, 10, cp_count).unwrap();
    eprintln!("parsed {} entries, next={}, cp_end={end}", pool.entries.len(), pool.next);
    let mut p = end + 6;
    let iface = u16::from_be_bytes([bytes[p], bytes[p+1]]);
    eprintln!("p={p} iface={iface}");
    p += 2 + 2 * iface as usize;
    eprintln!("after iface p={p}");
    let fields = u16::from_be_bytes([bytes[p], bytes[p+1]]);
    eprintln!("fields={fields}");
}

#[cfg(test)]
mod tests {
    use crate::classfile::*;

    const REAL: &[u8] = include_bytes!("../tests/fixtures/SingleUserAreaMap.class");

    #[test]
    fn patch_roundtrip() {
        let patched = patch_update(REAL).expect("patch");
        assert!(patched.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE]));
        // header preserved from the source class (kernel is Java 21 = major 65)
        assert_eq!(patched[..8], REAL[..8]);

        let cp_count = u16::from_be_bytes([patched[8], patched[9]]);
        let (pool, cp_end) = Pool::parse(&patched, 10, cp_count).expect("cp parse");
        assert!(cp_count > 100, "pool grew with appended entries");

        let this_idx = u16::from_be_bytes([patched[cp_end + 2], patched[cp_end + 3]]);
        let (_, tag, payload) = pool
            .entries
            .iter()
            .find(|(i, _, _)| *i == this_idx)
            .expect("this_class entry");
        assert_eq!(*tag, TAG_CLASS);
        let name_utf8 = u16::from_be_bytes([payload[0], payload[1]]);
        assert_eq!(pool.utf8_value(name_utf8).as_deref(), Some(MAP_CLASS));

        // walk members to the update method and check its Code attribute
        let mut p = cp_end + 6;
        let iface_count = usize::from(u16::from_be_bytes([patched[p], patched[p + 1]]));
        p += 2 + 2 * iface_count;
        let fields_count = usize::from(u16::from_be_bytes([patched[p], patched[p + 1]]));
        p += 2;
        for _ in 0..fields_count {
            p += 6;
            let attr_count = usize::from(u16::from_be_bytes([patched[p], patched[p + 1]]));
            p += 2;
            for _ in 0..attr_count {
                let len = u32::from_be_bytes([patched[p + 2], patched[p + 3], patched[p + 4], patched[p + 5]]);
                p += 6 + len as usize;
            }
        }
        let methods_count = usize::from(u16::from_be_bytes([patched[p], patched[p + 1]]));
        p += 2;
        let mut found_update = false;
        let mut found_code = false;
        for _ in 0..methods_count {
            let name_idx = u16::from_be_bytes([patched[p + 2], patched[p + 3]]);
            let desc_idx = u16::from_be_bytes([patched[p + 4], patched[p + 5]]);
            let attr_count = usize::from(u16::from_be_bytes([patched[p + 6], patched[p + 7]]));
            p += 8;
            let mut code_len = 0usize;
            let mut stackmap = false;
            for _ in 0..attr_count {
                let aname_idx = u16::from_be_bytes([patched[p], patched[p + 1]]);
                let alen = u32::from_be_bytes([patched[p + 2], patched[p + 3], patched[p + 4], patched[p + 5]]) as usize;
                if pool.utf8_value(aname_idx).as_deref() == Some("Code") {
                    code_len = u32::from_be_bytes([patched[p + 10], patched[p + 11], patched[p + 12], patched[p + 13]]) as usize;
                    // StackMapTable lives inside the Code attribute
                    let mut q = p + 14 + code_len; // after code bytes
                    let exc = usize::from(u16::from_be_bytes([patched[q], patched[q + 1]]));
                    q += 2 + 8 * exc;
                    let inner_attrs = usize::from(u16::from_be_bytes([patched[q], patched[q + 1]]));
                    q += 2;
                    for _ in 0..inner_attrs {
                        let ia_name = u16::from_be_bytes([patched[q], patched[q + 1]]);
                        let ia_len = u32::from_be_bytes([patched[q + 2], patched[q + 3], patched[q + 4], patched[q + 5]]) as usize;
                        if pool.utf8_value(ia_name).as_deref() == Some("StackMapTable") {
                            stackmap = true;
                        }
                        q += 6 + ia_len;
                    }
                }
                p += 6 + alen;
            }
            if pool.utf8_value(name_idx).as_deref() == Some("update")
                && pool.utf8_value(desc_idx).as_deref() == Some("(III)Z")
            {
                found_update = true;
                assert_eq!(code_len, 82, "patched update body is 82 bytes");
                assert!(stackmap, "update carries StackMapTable");
                found_code = true;
            }
        }
        assert!(found_update && found_code);

        let out = std::path::Path::new("/tmp/opencode/patchcheck/patched.class");
        std::fs::write(out, &patched).expect("dump patched class");
        eprintln!("wrote {} bytes to {}", patched.len(), out.display());
    }

    #[test]
    fn rejects_garbage() {
        let result = patch_update(&[0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52, 0, 5, 1, 2, 3, 4]);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod dbg2 {
    use super::*;

    #[test]
    fn dbg_parse2() {
        let bytes = include_bytes!("../tests/fixtures/SingleUserAreaMap.class");
        let cp_count = u16::from_be_bytes([bytes[8], bytes[9]]);
        let (pool, end) = Pool::parse(bytes, 10, cp_count).unwrap_or_else(|| panic!("parse fail"));
        eprintln!("entries={} next={} cp_end={end} cp_count={cp_count}", pool.entries.len(), pool.next);
        let this_idx = u16::from_be_bytes([bytes[end + 2], bytes[end + 3]]);
        eprintln!("this_class_idx={this_idx}");
        let found = pool.entries.iter().find(|(i, _, _)| *i == this_idx).cloned();
        let (_, t, p) = found.expect("this_class entry");
        eprintln!("this class entry tag={t} payload={p:02x?}");
        let name_utf8 = u16::from_be_bytes([p[0], p[1]]);
        eprintln!("this_name = {:?}", pool.utf8_value(name_utf8));
        let mut p2 = end + 6;
        let iface = u16::from_be_bytes([bytes[p2], bytes[p2 + 1]]);
        p2 += 2 + 2 * usize::from(iface);
        let fields = u16::from_be_bytes([bytes[p2], bytes[p2 + 1]]);
        eprintln!("iface={iface} fields={fields}");
    }
}

#[cfg(test)]
mod dbg3 {
    use super::*;

    #[test]
    fn dbg_methods() {
        let bytes = include_bytes!("../tests/fixtures/SingleUserAreaMap.class");
        let layout = parse_layout(bytes).unwrap();
        let mut p = layout.methods_start;
        let count = usize::from(u16::from_be_bytes([bytes[p], bytes[p + 1]]));
        eprintln!("methods_count={count} at offset {p}");
        p += 2;
        for mi in 0..count {
            let access = u16::from_be_bytes([bytes[p], bytes[p + 1]]);
            let n = u16::from_be_bytes([bytes[p + 2], bytes[p + 3]]);
            let d = u16::from_be_bytes([bytes[p + 4], bytes[p + 5]]);
            let attr_count = usize::from(u16::from_be_bytes([bytes[p + 6], bytes[p + 7]]));
            eprintln!(
                "m{mi}: access=0x{access:04x} name={} desc={} attrs={attr_count}",
                layout.pool.utf8_value(n).unwrap_or_default(),
                layout.pool.utf8_value(d).unwrap_or_default()
            );
            p += 8;
            for _ in 0..attr_count {
                let aname = u16::from_be_bytes([bytes[p], bytes[p + 1]]);
                let alen = u32::from_be_bytes([bytes[p + 2], bytes[p + 3], bytes[p + 4], bytes[p + 5]]);
                eprintln!("   attr {} len={alen}", layout.pool.utf8_value(aname).unwrap_or_default());
                p += 6 + alen as usize;
            }
        }
    }
}
