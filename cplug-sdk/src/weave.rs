//! Pure-Rust classfile patching — transform bytes offline, no JVM required.
//!
//! Two operations:
//! - `redirect_calls`: retarget every invocation of a given
//!   (class, name, desc) to a new (class, name) — index-only patch, the
//!   instruction stream never moves.
//! - `insert_call_at_start`: prepend `ldc "msg"; invokestatic target; pop*`
//!   to a method — a truly net-zero-stack prefix (the ldc operand and any
//!   target result are popped before the original body runs, so max_stack
//!   needs no bump and stack-map frames need no stack edits); all
//!   offset-sensitive structures (switch tables, exception table, stack map
//!   frames, line numbers, local variable tables) are shifted by the
//!   insertion delta.
//!
//! Frame-verifier note: inserted code is stack-neutral and stack map entries
//! move uniformly with the code, so a class that verified before still
//! verifies after. The first StackMapTable frame's offset_delta is relative
//! to code start and shifts by the delta; later deltas are frame-to-frame and
//! are unchanged.

pub fn insert_call_at_start(
    mut bytes: Vec<u8>,
    method_name: &str,
    method_desc: &str,
    target_class: &str,
    target_name: &str,
    target_desc: &str,
    msg: Option<&str>,
) -> Result<Vec<u8>, String> {
    let cp = Cp::parse(&bytes)?;
    let region = find_method_code(&bytes, &cp, method_name, method_desc)?;

    // Build the CP additions and the prefix referencing them by future index.
    // Note: ldc_w requires a CONSTANT_String (tag 8), not a raw Utf8 — the
    // utf8 is inserted first and a String entry points at it.
    let mut additions: Vec<u8> = Vec::new();
    let mut next = cp.count - 1; // last occupied cp index (cp.count includes slot 0)
    let mut ldc_idx = 0u16;
    if let Some(m) = msg {
        let utf8_idx = next + 1;
        push_utf8(&mut additions, m);
        next += 1;
        ldc_idx = next + 1;
        additions.push(8); // String -> msg utf8
        additions.extend_from_slice(&utf8_idx.to_be_bytes());
        next += 1;
    }
    let class_name_idx = next + 1;
    push_utf8(&mut additions, target_class);
    next += 1;
    let name_idx = next + 1;
    push_utf8(&mut additions, target_name);
    next += 1;
    let desc_idx = next + 1;
    push_utf8(&mut additions, target_desc);
    next += 1;
    let class_idx = next + 1;
    additions.push(7); // Class -> target class name utf8
    additions.extend_from_slice(&class_name_idx.to_be_bytes());
    next += 1;
    let nat_idx = next + 1;
    additions.push(12); // NameAndType -> name, desc
    additions.extend_from_slice(&name_idx.to_be_bytes());
    additions.extend_from_slice(&desc_idx.to_be_bytes());
    next += 1;
    let methodref_idx = next + 1;
    additions.push(10); // Methodref -> class, nat
    additions.extend_from_slice(&class_idx.to_be_bytes());
    additions.extend_from_slice(&nat_idx.to_be_bytes());

    let mut prefix: Vec<u8> = Vec::new();
    if msg.is_some() {
        prefix.push(0x13); // ldc_w (u2 index — safe for any CP size)
        prefix.extend_from_slice(&ldc_idx.to_be_bytes());
    }
    prefix.push(0xb8); // invokestatic
    prefix.extend_from_slice(&methodref_idx.to_be_bytes());
    let ret_void = target_desc.ends_with(")V");
    if !ret_void {
        prefix.push(0x57); // pop: drop the target's result
    }
    if msg.is_some() {
        prefix.push(0x57); // pop: drop the ldc operand. The prefix must be
                           // fully stack-neutral — the original body then runs
                           // on an empty stack, so max_stack stays untouched
                           // and stack-map frames need no stack edits.
    }
    let delta = prefix.len();

    // Rebuild the code array with shifted switch targets.
    let old_code = bytes[region.code_start..region.code_end].to_vec();
    let new_code = shift_code(&old_code, &prefix)?;

    // ---- splices, deepest first ----
    // 1) replace the code array (same length as old + delta)
    let splice_pos = region.code_start;
    bytes.splice(
        splice_pos..splice_pos + old_code.len(),
        new_code.iter().copied(),
    );
    // 2) fix lengths + max_stack inside the Code attribute (offsets unchanged:
    //    the code array grew in place, everything after it shifted by delta)
    write_u16(&mut bytes, region.max_stack_pos, region.max_stack.max(1));
    write_u32(
        &mut bytes,
        region.code_len_pos,
        (old_code.len() + delta) as u32,
    );
    write_u32(
        &mut bytes,
        region.attr_len_pos,
        region.attr_len + delta as u32,
    );
    // 3) exception table + nested attributes (shifted by delta)
    let exc_start = region.exc_start + delta;
    let exc_count = u16at(&bytes, exc_start) as usize;
    for i in 0..exc_count {
        let p = exc_start + 2 + i * 8;
        for off in 0..3 {
            let v = u16at(&bytes, p + off * 2) + delta as u16;
            write_u16(&mut bytes, p + off * 2, v);
        }
    }
    let attrs_start = region.attrs_start + delta;
    shift_code_attributes(&mut bytes[..], attrs_start, delta)?;
    // 4) append CP additions (shifts nothing above: cp section precedes)
    let cp_end = cp.end;
    bytes.splice(cp_end..cp_end, additions.iter().copied());
    write_u16(
        &mut bytes,
        cp.count_pos,
        cp.count + additions_count(&additions),
    );

    Ok(bytes)
}

fn additions_count(additions: &[u8]) -> u16 {
    let mut n = 0u16;
    let mut i = 0usize;
    while i < additions.len() {
        let tag = additions[i];
        i += 1;
        match tag {
            1 => {
                let l = u16at(additions, i) as usize;
                i += 2 + l;
            }
            7 | 8 | 16 => i += 2,
            10 | 11 | 12 | 17 | 18 => i += 4,
            15 => i += 3,
            _ => break,
        }
        n += 1;
    }
    n
}

/// Prepend `prefix` to a code array and re-emit with fresh switch padding,
/// shifting every absolute offset (switch targets) by `prefix.len()`.
/// Branch operands are relative — unchanged.
fn shift_code(code: &[u8], prefix: &[u8]) -> Result<Vec<u8>, String> {
    let delta = prefix.len();
    let instrs = walk(code)?;
    let mut out = Vec::with_capacity(code.len() + delta);
    out.extend_from_slice(prefix);
    for (pc, len) in instrs {
        let op = code[pc];
        match op {
            0xaa | 0xab => emit_switch(&mut out, op, &code[pc..pc + len], pc, delta)?,
            _ => out.extend_from_slice(&code[pc..pc + len]),
        }
    }
    Ok(out)
}

/// Re-emit a switch instruction at the new position: fresh 4-byte padding,
/// absolute offsets shifted by `delta`. `raw` is the original instruction
/// (opcode..opcode+len) at old pc `pc_old`.
fn emit_switch(
    out: &mut Vec<u8>,
    op: u8,
    raw: &[u8],
    pc_old: usize,
    delta: usize,
) -> Result<(), String> {
    let old_pad = switch_pad(pc_old);
    let body = &raw[1 + old_pad..];
    let new_pad = (4 - (out.len() + 1) % 4) % 4;
    out.push(op);
    out.extend(std::iter::repeat_n(0u8, new_pad));
    let shifted = |off: &[u8]| -> Vec<u8> {
        let v = i32::from_be_bytes(off.try_into().unwrap()) + delta as i32;
        v.to_be_bytes().to_vec()
    };
    match op {
        0xaa => {
            if body.len() < 12 {
                return Err("tableswitch too short".into());
            }
            let low = i32::from_be_bytes(body[4..8].try_into().unwrap());
            let high = i32::from_be_bytes(body[8..12].try_into().unwrap());
            let n = (high - low + 1).max(0) as usize;
            if body.len() < 12 + n * 4 {
                return Err("tableswitch entries truncated".into());
            }
            out.extend_from_slice(&shifted(&body[0..4])); // default
            out.extend_from_slice(&body[4..8]); // low (bound, not an offset)
            out.extend_from_slice(&body[8..12]); // high (bound, not an offset)
            for chunk in body[12..12 + n * 4].chunks_exact(4) {
                out.extend_from_slice(&shifted(chunk));
            }
        }
        0xab => {
            if body.len() < 8 {
                return Err("lookupswitch too short".into());
            }
            let n = u32::from_be_bytes(body[4..8].try_into().unwrap()) as usize;
            if body.len() < 8 + n * 8 {
                return Err("lookupswitch pairs truncated".into());
            }
            out.extend_from_slice(&shifted(&body[0..4])); // default
            out.extend_from_slice(&body[4..8]); // npairs (not an offset)
            for pair in body[8..8 + n * 8].chunks_exact(8) {
                out.extend_from_slice(&pair[0..4]); // match key
                out.extend_from_slice(&shifted(&pair[4..8])); // jump offset
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// Shift every attribute stored inside a Code attribute: StackMapTable
/// (first frame's offset_delta), LineNumberTable, LocalVariableTable,
/// LocalVariableTypeTable. Returns the position after the last attribute.
fn shift_code_attributes(bytes: &mut [u8], attrs_start: usize, delta: usize) -> Result<(), String> {
    let count = u16at(bytes, attrs_start) as usize;
    let mut p = attrs_start + 2;
    for _ in 0..count {
        let name_idx = u16at(bytes, p);
        let len = u32at(bytes, p + 2) as usize;
        let body = p + 6;
        let name = utf8_at(bytes, name_idx).unwrap_or_default();
        match name.as_str() {
            "StackMapTable" => {
                shift_stack_map(bytes, body, len, delta)?;
            }
            "LineNumberTable" => {
                let n = u16at(bytes, body) as usize;
                for i in 0..n {
                    let q = body + 2 + i * 4;
                    let v = u16at(bytes, q) + delta as u16;
                    write_u16(bytes, q, v);
                }
            }
            "LocalVariableTable" | "LocalVariableTypeTable" => {
                let n = u16at(bytes, body) as usize;
                for i in 0..n {
                    let q = body + 2 + i * 10;
                    let v = u16at(bytes, q) + delta as u16;
                    write_u16(bytes, q, v);
                }
            }
            _ => {}
        }
        p = body + len;
    }
    Ok(())
}

fn shift_stack_map(bytes: &mut [u8], body: usize, len: usize, delta: usize) -> Result<(), String> {
    let n = u16at(bytes, body) as usize;
    let mut p = body + 2;
    let end = body + len;
    for i in 0..n {
        if p >= end {
            return Err("StackMapTable truncated".into());
        }
        let tag = bytes[p];
        match tag {
            0..=63 => p += 1,                                  // same_frame
            64..=127 => p += 1 + vti_len(&bytes[p + 1..end])?, // same_locals_1_stack_item
            247 => {
                if i == 0 {
                    let v = u16at(bytes, p + 1);
                    write_u16(bytes, p + 1, v + delta as u16);
                }
                p += 3 + vti_len(&bytes[p + 3..end])?;
            }
            248..=251 => {
                if i == 0 {
                    let v = u16at(bytes, p + 1);
                    write_u16(bytes, p + 1, v + delta as u16);
                }
                p += 3;
            }
            252..=254 => {
                if i == 0 {
                    let v = u16at(bytes, p + 1);
                    write_u16(bytes, p + 1, v + delta as u16);
                }
                p += 3;
                let locals = tag as usize - 251;
                for _ in 0..locals {
                    p += vti_len(&bytes[p..end])?;
                }
            }
            255 => {
                if i == 0 {
                    let v = u16at(bytes, p + 1);
                    write_u16(bytes, p + 1, v + delta as u16);
                }
                p += 3;
                let loc = u16at(bytes, p) as usize;
                p += 2;
                for _ in 0..loc {
                    p += vti_len(&bytes[p..end])?;
                }
                let stk = u16at(bytes, p) as usize;
                p += 2;
                for _ in 0..stk {
                    p += vti_len(&bytes[p..end])?;
                }
            }
            _ => return Err(format!("bad frame tag {tag}")),
        }
    }
    Ok(())
}

fn vti_len(b: &[u8]) -> Result<usize, String> {
    if b.is_empty() {
        return Err("vti truncated".into());
    }
    match b[0] {
        0..=6 => Ok(1), // Top..Double
        7 | 8 => Ok(3), // Object(2), Uninitialized(2)
        _ => Err(format!("bad vti tag {}", b[0])),
    }
}

// ---------------------------------------------------------------------------
// redirect
// ---------------------------------------------------------------------------

/// Retarget every invocation (invokestatic/invokevirtual/invokespecial/
/// invokeinterface) of `old_class.old_name(old_desc)` to
/// `new_class.new_name`. Returns the number of patched call sites. Modifies
/// `bytes` in place.
pub fn redirect_calls(
    bytes: &mut Vec<u8>,
    old_class: &str,
    old_name: &str,
    old_desc: &str,
    new_class: &str,
    new_name: &str,
) -> Result<usize, String> {
    let cp = Cp::parse(bytes)?;
    let mut old_mrefs: Vec<u16> = Vec::new();
    let mut needs_iface = false;
    for (idx, e) in cp.entries.iter().enumerate() {
        if e.tag == 10 || e.tag == 11 {
            if let Ok((c, n, d)) = methodref_target(bytes, &cp, (idx + 1) as u16) {
                if c == old_class && n == old_name && d == old_desc {
                    old_mrefs.push((idx + 1) as u16);
                    if e.tag == 11 {
                        needs_iface = true;
                    }
                }
            }
        }
    }
    if old_mrefs.is_empty() {
        return Ok(0);
    }

    // append new CP entries: Class, NAT, Methodref (+InterfaceMethodref)
    let mut additions: Vec<u8> = Vec::new();
    let name_idx = cp.count; // first new slot: new_class utf8
    let method_name_idx = name_idx + 1; // new_name utf8
    let desc_idx = name_idx + 2; // old_desc utf8
    let class_idx = name_idx + 3; // Class
    let nat_idx = name_idx + 4; // NameAndType
    let mref_idx = name_idx + 5; // Methodref
    let iface_idx = if needs_iface { name_idx + 6 } else { 0 };
    push_utf8(&mut additions, new_class);
    push_utf8(&mut additions, new_name);
    push_utf8(&mut additions, old_desc);
    additions.push(7); // Class -> new_class utf8
    additions.extend_from_slice(&name_idx.to_be_bytes());
    additions.push(12); // NameAndType -> new_name, desc
    additions.extend_from_slice(&method_name_idx.to_be_bytes());
    additions.extend_from_slice(&desc_idx.to_be_bytes());
    additions.push(10); // Methodref -> class, nat
    additions.extend_from_slice(&class_idx.to_be_bytes());
    additions.extend_from_slice(&nat_idx.to_be_bytes());
    if needs_iface {
        additions.push(11); // InterfaceMethodref -> class, nat
        additions.extend_from_slice(&class_idx.to_be_bytes());
        additions.extend_from_slice(&nat_idx.to_be_bytes());
    }

    // collect call-site operand positions (absolute), then splice CP, then patch
    let mut sites: Vec<usize> = Vec::new();
    let mut is_iface: Vec<bool> = Vec::new();
    walk_methods(bytes, &cp, |code_start, code_end| {
        let code = &bytes[code_start..code_end];
        if let Ok(instrs) = walk(code) {
            for (pc, len) in instrs {
                let op = code[pc];
                if matches!(op, 0xb6..=0xb8) {
                    let idx = u16at(code, pc + 1);
                    if old_mrefs.contains(&idx) {
                        sites.push(code_start + pc + 1);
                        is_iface.push(false);
                    }
                } else if op == 0xb9 {
                    let idx = u16at(code, pc + 1);
                    if old_mrefs.contains(&idx) {
                        sites.push(code_start + pc + 1);
                        is_iface.push(true);
                    }
                }
                let _ = len;
            }
        }
    });

    let cp_end = cp.end;
    bytes.splice(cp_end..cp_end, additions.iter().copied());
    write_u16(bytes, cp.count_pos, cp.count + additions_count(&additions));
    let shift = additions.len();
    for (i, pos) in sites.iter().enumerate() {
        let idx = if is_iface[i] { iface_idx } else { mref_idx };
        write_u16(bytes, *pos + shift, idx);
    }

    Ok(sites.len())
}

// ---------------------------------------------------------------------------
// parsing helpers
// ---------------------------------------------------------------------------

fn u16at(b: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([b[i], b[i + 1]])
}

fn u32at(b: &[u8], i: usize) -> u32 {
    u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

fn write_u16(b: &mut [u8], i: usize, v: u16) {
    b[i..i + 2].copy_from_slice(&v.to_be_bytes());
}

fn write_u32(b: &mut [u8], i: usize, v: u32) {
    b[i..i + 4].copy_from_slice(&v.to_be_bytes());
}

fn push_utf8(c: &mut Vec<u8>, s: &str) {
    c.push(1);
    let b = s.as_bytes();
    c.extend_from_slice(&(b.len() as u16).to_be_bytes());
    c.extend_from_slice(b);
}

struct CpEntry {
    tag: u8,
    start: usize,
}

struct Cp {
    count_pos: usize,
    count: u16,
    end: usize,
    entries: Vec<CpEntry>,
}

impl Cp {
    fn parse(b: &[u8]) -> Result<Cp, String> {
        if b.len() < 10 || u32at(b, 0) != 0xCAFE_BABE {
            return Err("not a class file".into());
        }
        let count_pos = 8;
        let count = u16at(b, count_pos);
        let mut p = 10;
        let mut entries = Vec::with_capacity(count as usize - 1);
        let mut idx = 1;
        while idx < count {
            if p >= b.len() {
                return Err("cp truncated".into());
            }
            let tag = b[p];
            let start = p;
            p += 1;
            match tag {
                1 => {
                    let l = u16at(b, p) as usize;
                    p += 2 + l;
                }
                7 | 8 | 16 | 19 | 20 => p += 2,
                9 | 10 | 11 | 12 | 17 | 18 => p += 4,
                15 => p += 3,
                3 | 4 => p += 4,
                5 | 6 => {
                    p += 8;
                    entries.push(CpEntry { tag: 0, start: 0 }); // phantom slot
                    idx += 1;
                }
                _ => return Err(format!("bad cp tag {tag} at {start}")),
            }
            if p > b.len() {
                return Err("cp overrun".into());
            }
            entries.push(CpEntry { tag, start });
            idx += 1;
        }
        Ok(Cp {
            count_pos,
            count,
            end: p,
            entries,
        })
    }

    fn utf8<'a>(&self, b: &'a [u8], idx: u16) -> Result<&'a str, String> {
        let e = self.entry(idx)?;
        if e.tag != 1 {
            return Err("not utf8".into());
        }
        let l = u16at(b, e.start + 1) as usize;
        std::str::from_utf8(&b[e.start + 3..e.start + 3 + l]).map_err(|_| "bad utf8".into())
    }

    fn class_name(&self, b: &[u8], idx: u16) -> Result<String, String> {
        let e = self.entry(idx)?;
        if e.tag != 7 {
            return Err("not class".into());
        }
        let name_idx = u16at(b, e.start + 1);
        Ok(self.utf8(b, name_idx)?.to_string())
    }

    fn nat(&self, b: &[u8], idx: u16) -> Result<(String, String), String> {
        let e = self.entry(idx)?;
        if e.tag != 12 {
            return Err("not nameandtype".into());
        }
        let name_idx = u16at(b, e.start + 1);
        let desc_idx = u16at(b, e.start + 3);
        Ok((
            self.utf8(b, name_idx)?.to_string(),
            self.utf8(b, desc_idx)?.to_string(),
        ))
    }

    fn entry(&self, idx: u16) -> Result<&CpEntry, String> {
        let i = idx as usize;
        if i == 0 || i > self.entries.len() {
            return Err(format!("cp index {idx} out of range"));
        }
        Ok(&self.entries[i - 1])
    }
}

fn methodref_target(b: &[u8], cp: &Cp, idx: u16) -> Result<(String, String, String), String> {
    let e = cp.entry(idx)?;
    if e.tag != 10 && e.tag != 11 {
        return Err("not methodref".into());
    }
    let class_idx = u16at(b, e.start + 1);
    let nat_idx = u16at(b, e.start + 3);
    let (name, desc) = cp.nat(b, nat_idx)?;
    let class = cp.class_name(b, class_idx)?;
    Ok((class, name, desc))
}

fn utf8_at(b: &[u8], idx: u16) -> Option<String> {
    let cp = Cp::parse(b).ok()?;
    cp.utf8(b, idx).ok().map(str::to_string)
}

// ---------------------------------------------------------------------------
// structural walkers
// ---------------------------------------------------------------------------

/// Parse every instruction boundary: Vec of (pc, len). `code` is the raw code
/// array.
fn walk(code: &[u8]) -> Result<Vec<(usize, usize)>, String> {
    let mut out = Vec::new();
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let len = instr_len(code, pc, op)?;
        out.push((pc, len));
        pc += len;
    }
    Ok(out)
}

fn instr_len(code: &[u8], pc: usize, op: u8) -> Result<usize, String> {
    let n = match op {
        0x00..=0x0f | 0x1a..=0x35 | 0x3b..=0x4d | 0x4e..=0x83 | 0x85..=0x98 => 1,
        0x10 | 0x12 | 0x15..=0x19 | 0x36..=0x3a | 0xa9 | 0xbc | 0xc2 | 0xc3 => 2,
        0x11
        | 0x13
        | 0x14
        | 0x84
        | 0x99..=0xa8
        | 0xb2..=0xb8
        | 0xbb
        | 0xbd
        | 0xc0
        | 0xc1
        | 0xc6
        | 0xc7 => 3,
        0xba | 0xc5 => 4,
        0xb9 | 0xc8 | 0xc9 => 5,
        0xc4 => {
            if pc + 1 < code.len() && code[pc + 1] == 0x84 {
                6
            } else {
                4
            }
        }
        0xaa | 0xab => {
            let pad = switch_pad(pc);
            let body = &code[pc + 1 + pad..];
            match op {
                0xaa => {
                    if body.len() < 12 {
                        return Err("tableswitch truncated".into());
                    }
                    let low = i32::from_be_bytes(body[4..8].try_into().unwrap());
                    let high = i32::from_be_bytes(body[8..12].try_into().unwrap());
                    1 + pad + 12 + (high - low + 1).max(0) as usize * 4
                }
                0xab => {
                    if body.len() < 8 {
                        return Err("lookupswitch truncated".into());
                    }
                    let n = u32::from_be_bytes(body[4..8].try_into().unwrap()) as usize;
                    1 + pad + 8 + n * 8
                }
                _ => unreachable!(),
            }
        }
        _ => 1, // reserved / breakpoint
    };
    Ok(n)
}

fn switch_pad(pc: usize) -> usize {
    (4 - (pc + 1) % 4) % 4
}

/// Find the Code attribute of the method with `name`/`desc`. Returns the
/// absolute byte offsets needed for splicing.
struct CodeRegion {
    max_stack_pos: usize,
    max_stack: u16,
    code_len_pos: usize,
    code_start: usize,
    code_end: usize,
    attr_len_pos: usize,
    attr_len: u32,
    exc_start: usize,
    attrs_start: usize,
}

fn find_method_code(b: &[u8], cp: &Cp, name: &str, desc: &str) -> Result<CodeRegion, String> {
    let mut p = cp.end;
    p += 6; // access_flags, this_class, super_class
    let ifaces = u16at(b, p) as usize;
    p += 2 + ifaces * 2;
    p = skip_fields(b, p)?;
    let methods = u16at(b, p) as usize;
    p += 2;
    for _ in 0..methods {
        let p_name = p + 2;
        let p_desc = p + 4;
        let m_name = cp.utf8(b, u16at(b, p_name)).unwrap_or_default();
        let m_desc = cp.utf8(b, u16at(b, p_desc)).unwrap_or_default();
        let attr_count = u16at(b, p + 6) as usize;
        p += 8;
        for _ in 0..attr_count {
            let attr_name_idx = u16at(b, p);
            let attr_len = u32at(b, p + 2) as usize;
            let attr_body = p + 6;
            let a_name = cp.utf8(b, attr_name_idx).unwrap_or_default();
            if a_name == "Code" && m_name == name && m_desc == desc {
                let max_stack = u16at(b, attr_body);
                let code_len = u32at(b, attr_body + 4) as usize;
                let code_start = attr_body + 8;
                let code_end = code_start + code_len;
                let exc_start = code_end;
                let attrs_start = exc_start + 2 + u16at(b, exc_start) as usize * 8;
                return Ok(CodeRegion {
                    max_stack_pos: attr_body,
                    max_stack,
                    code_len_pos: attr_body + 4,
                    code_start,
                    code_end,
                    attr_len_pos: p + 2,
                    attr_len: attr_len as u32,
                    exc_start,
                    attrs_start,
                });
            }
            p = attr_body + attr_len;
        }
    }
    Err(format!("method {name}{desc} not found"))
}

fn skip_fields(b: &[u8], mut p: usize) -> Result<usize, String> {
    let fields = u16at(b, p) as usize;
    p += 2;
    for _ in 0..fields {
        let attr_count = u16at(b, p + 6) as usize;
        p += 8;
        for _ in 0..attr_count {
            let attr_len = u32at(b, p + 2) as usize;
            p += 6 + attr_len;
        }
    }
    Ok(p)
}

/// Call `f(code_start, code_end)` for every method's Code array.
fn walk_methods(b: &[u8], cp: &Cp, mut f: impl FnMut(usize, usize)) {
    let mut p = cp.end;
    p += 6;
    let ifaces = u16at(b, p) as usize;
    p += 2 + ifaces * 2;
    if let Ok(fp) = skip_fields(b, p) {
        p = fp;
    } else {
        return;
    }
    let methods = u16at(b, p) as usize;
    p += 2;
    for _ in 0..methods {
        let attr_count = u16at(b, p + 6) as usize;
        p += 8;
        for _ in 0..attr_count {
            let attr_name_idx = u16at(b, p);
            let attr_len = u32at(b, p + 2) as usize;
            let attr_body = p + 6;
            let a_name = cp.utf8(b, attr_name_idx).unwrap_or_default();
            if a_name == "Code" && attr_len >= 8 {
                let code_len = u32at(b, attr_body + 4) as usize;
                let code_start = attr_body + 8;
                f(code_start, code_start + code_len);
            }
            p = attr_body + attr_len;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn jar_class(entry: &str) -> Option<Vec<u8>> {
        let path = format!(
            "{}/../dist-paper/libs/paper-api-1.21.10.jar",
            env!("CARGO_MANIFEST_DIR")
        );
        let jar = std::fs::read(&path).ok()?;
        extract_from_jar(&jar, entry)
    }

    fn extract_from_jar(jar: &[u8], entry: &str) -> Option<Vec<u8>> {
        use std::io::Read;
        let n = jar.len();
        let mut eocd = None;
        let from = n.checked_sub(22)?;
        for i in (0..from).rev() {
            if &jar[i..i + 4] == b"PK\x05\x06" {
                eocd = Some(i);
                break;
            }
        }
        let i = eocd?;
        let cd_offset = u32::from_le_bytes(jar[i + 16..i + 20].try_into().ok()?) as usize;
        let entries = u16::from_le_bytes(jar[i + 10..i + 12].try_into().ok()?) as usize;
        let mut p = cd_offset;
        for _ in 0..entries {
            if &jar[p..p + 4] != b"PK\x01\x02" {
                return None;
            }
            let method = u16::from_le_bytes(jar[p + 10..p + 12].try_into().ok()?);
            let comp = u32::from_le_bytes(jar[p + 20..p + 24].try_into().ok()?) as usize;
            let name_len = u16::from_le_bytes(jar[p + 28..p + 30].try_into().ok()?) as usize;
            let extra_len = u16::from_le_bytes(jar[p + 30..p + 32].try_into().ok()?) as usize;
            let comment_len = u16::from_le_bytes(jar[p + 32..p + 34].try_into().ok()?) as usize;
            let lh = u32::from_le_bytes(jar[p + 42..p + 46].try_into().ok()?) as usize;
            if &jar[p + 46..p + 46 + name_len] == entry.as_bytes() {
                let lname = u16::from_le_bytes(jar[lh + 26..lh + 28].try_into().ok()?) as usize;
                let lextra = u16::from_le_bytes(jar[lh + 28..lh + 30].try_into().ok()?) as usize;
                let data = &jar[lh + 30 + lname + lextra..lh + 30 + lname + lextra + comp];
                return match method {
                    0 => Some(data.to_vec()),
                    8 => {
                        let mut dec = flate2::read::DeflateDecoder::new(data);
                        let mut out = Vec::new();
                        dec.read_to_end(&mut out).ok()?;
                        Some(out)
                    }
                    _ => None,
                };
            }
            p += 46 + name_len + extra_len + comment_len;
        }
        None
    }

    #[test]
    fn insert_into_runnable_ctor() {
        let bytes = crate::main_thread::runnable_class_bytes("dev/dist/SdkNativeRunnable");
        let patched = insert_call_at_start(
            bytes,
            "<init>",
            "()V",
            "java/lang/System",
            "gc",
            "()V",
            Some("hello from weave"),
        )
        .unwrap();
        // prefix = ldc_w(3) + invokestatic(3) + pop(1) = 7, target is ()V so
        // the result pop is skipped; the trailing pop clears the ldc operand.
        // cp additions: utf8 msg, String, utf8 class, utf8 name, utf8 desc,
        // Class, NAT, Methodref = 8
        let cp = Cp::parse(&patched).unwrap();
        assert_eq!(cp.count, 15 + 8);
        let region = find_method_code(&patched, &cp, "<init>", "()V").unwrap();
        assert_eq!(region.code_end - region.code_start, 5 + 7);
        let instrs = walk(&patched[region.code_start..region.code_end]).unwrap();
        assert_eq!(instrs[0], (0, 3)); // ldc_w
        assert_eq!(instrs[1], (3, 3)); // invokestatic
        assert_eq!(instrs[2], (6, 1)); // pop
        assert_eq!(instrs[3], (7, 1)); // aload_0
        assert_eq!(instrs[4], (8, 3)); // invokespecial
        assert_eq!(instrs[5], (11, 1)); // return
                                        // ldc_w must reference a CONSTANT_String (tag 8) — a raw Utf8
                                        // verifier-rejects ("Illegal type at constant pool entry").
        let ldc_idx = u16at(&patched, region.code_start + 1);
        let ldc_entry = cp.entry(ldc_idx).unwrap();
        assert_eq!(ldc_entry.tag, 8, "ldc operand must be a String constant");
        // invokestatic must reference the new methodref (last appended slot)
        let idx = u16at(&patched, region.code_start + 4);
        assert_eq!(idx, cp.count - 1);
        let (c, n, d) = methodref_target(&patched, &cp, idx).unwrap();
        assert_eq!(
            (c.as_str(), n.as_str(), d.as_str()),
            ("java/lang/System", "gc", "()V")
        );
        // re-walk everything structurally clean
        walk(&patched[region.code_start..region.code_end]).unwrap();
    }

    #[test]
    fn redirect_runnable_ctor_call() {
        let mut bytes = crate::main_thread::runnable_class_bytes("dev/dist/SdkNativeRunnable");
        let n = redirect_calls(
            &mut bytes,
            "java/lang/Object",
            "<init>",
            "()V",
            "let/me/Die",
            "plz",
        )
        .unwrap();
        assert_eq!(n, 1);
        let cp = Cp::parse(&bytes).unwrap();
        // find the patch site: <init> method's invokespecial operand
        let region = find_method_code(&bytes, &cp, "<init>", "()V").unwrap();
        let code = &bytes[region.code_start..region.code_end];
        // invokespecial at offset 1, operand at 2..4
        assert_eq!(code[1], 0xb7);
        let idx = u16at(code, 2);
        let (c, n, d) = methodref_target(&bytes, &cp, idx).unwrap();
        assert_eq!(
            (c.as_str(), n.as_str(), d.as_str()),
            ("let/me/Die", "plz", "()V")
        );
    }

    #[test]
    fn insert_into_real_bukkit_getversion() {
        let Some(bytes) = jar_class("org/bukkit/Bukkit.class") else {
            eprintln!("paper-api jar not present, skipping");
            return;
        };
        let patched = insert_call_at_start(
            bytes,
            "getVersion",
            "()Ljava/lang/String;",
            "java/lang/System",
            "gc",
            "()V",
            Some("weave-test"),
        )
        .expect("patch getVersion");
        let cp = Cp::parse(&patched).unwrap();
        let region = find_method_code(&patched, &cp, "getVersion", "()Ljava/lang/String;").unwrap();
        walk(&patched[region.code_start..region.code_end]).unwrap();
        // dump + javap structural validation (skip if no javap)
        let dir = std::env::temp_dir().join(format!("cplug-weave-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Bukkit.class"), &patched).unwrap();
        let javap = std::process::Command::new("javap")
            .args(["-v", "-p"])
            .arg(dir.join("Bukkit.class"))
            .output();
        match javap {
            Ok(out) => {
                let ok = out.status.success();
                assert!(
                    ok,
                    "javap rejected patched class:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                );
                let text = String::from_utf8_lossy(&out.stdout);
                assert!(
                    text.contains("weave-test"),
                    "ldc string not found in javap output"
                );
                assert!(
                    text.contains("java/lang/System.gc"),
                    "target call not found in javap output"
                );
            }
            Err(_) => eprintln!("javap not available, skipping structural validation"),
        }
    }
}
