//! Brick 1: transform engine — scriptable bytecode transformation.
//!
//! Modules register transformation rules that fire on class load (and
//! retransform). A rule matches a class by name pattern and optionally a
//! method by name+descriptor, then applies a transformation script: the
//! engine inserts a static-method call at method entry (`MethodEntry`) or
//! before a matched call site (`BeforeCall`), which covers the
//! overwhelming majority of fork-style patches (probes, guards, counters).
//! Full method-body rewriting is out of scope for the MVP.
//!
//! # Strategy
//!
//! The engine is a minimal, dependency-free class-file parser (JVMS
//! chapter 4) that performs **surgical byte-level edits** on the original
//! buffer — the ASM model — instead of re-serialising a parsed AST:
//! everything except the patched regions is preserved verbatim, which is
//! the safest possible change for code running inside a live JVM.
//! Research before writing compared the Rust crates that can *write*
//! class files: [`ristretto_classfile`](https://lib.rs/crates/ristretto_classfile)
//! (maintained, read+write+verify, the strongest candidate),
//! [`java_asm`](https://github.com/zsqw123/rust-java-asm) (writer "Not
//! Started"), [`classfile-rs`](https://github.com/x4e/classfile-rs) (writer
//! TODO), [`classfile-parser`](https://lib.rs/crates/classfile-parser)
//! (read-only). For the narrow operation needed here — append
//! constant-pool entries, prepend `invokestatic`, fix the affected offsets
//! — a std-only parser with explicit, tested offset fixups is the smallest
//! trustworthy surface: zero new dependencies in a JVM-hosted cdylib, and
//! untouched bytes stay byte-identical (a full AST round-trip would
//! re-serialise the whole class through third-party code).
//!
//! Offsets are fixed when instructions are inserted (per JVMS 4.7.x):
//! branch operands (`if*`, `goto`, `goto_w`, `jsr`, `jsr_w`, `tableswitch`,
//! `lookupswitch`), the `Code` exception table (`start_pc`/`end_pc`/
//! `handler_pc`), `StackMapTable` frame deltas, `LineNumberTable` and
//! `LocalVariableTable`/`LocalVariableTypeTable` ranges. `StackMapTable`
//! frames are delta-encoded, so only the first frame past the insertion
//! point changes; its encoding is re-chosen (`same_frame` →
//! `same_frame_extended`, `same_locals_1_stack_item_frame` →
//! `..._extended`) when the delta overflows the inline 6-bit field.
//!
//! # MVP limitations
//!
//! - The injected helper must be a `static` method with descriptor `()V`
//!   (no arguments, no return value), e.g. `dev.crussty.hooks.TickHook.onEntry`.
//! - Code-level type annotations (`RuntimeVisibleTypeAnnotations` /
//!   `RuntimeInvisibleTypeAnnotations` inside `Code`) are not
//!   offset-fixed. The verifier ignores them; only reflection-based
//!   tooling would see stale targets.
//! - A branch (or exception handler) targeting exactly the insertion point
//!   keeps pointing at it, i.e. at the inserted hook — this preserves the
//!   "hook runs first" semantics (never emitted by javac for entry
//!   insertion, but possible for `BeforeCall` on loop-head call sites).
//! - `BeforeCall` targets are matched as `owner.name:descriptor` (internal
//!   names); a target containing no `/` matches any owner with that
//!   name+descriptor.
//! - Unknown constant-pool tags and malformed structures abort with `Err`
//!   (never panic); a class that parses but yields no edits passes through
//!   as `Ok(None)`.
//! - Instrumentation is idempotent per helper: re-running `apply` (e.g.
//!   JVMTI retransformation) does not double-instrument.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Where to inject the instrumented call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Injection {
    /// Before the first instruction of a method body.
    MethodEntry,
    /// Before every call site whose resolved target
    /// (`owner.name:descriptor`, internal names) matches the pattern.
    BeforeCall(String),
}

/// One transformation rule.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Class internal name pattern, e.g. "net/minecraft/server/level/ServerLevel" or "a/b/*"
    pub class_pattern: String,
    /// Optional method name filter ("*" = any)
    pub method: String,
    /// Optional method descriptor filter ("*" = any), e.g. "()V"
    pub descriptor: String,
    /// What to inject
    pub injection: Injection,
    /// Fully-qualified static helper, e.g. "dev.crussty.hooks.TickHook.onEntry"
    pub helper: String,
}

impl Rule {
    pub fn new(
        class_pattern: impl Into<String>,
        method: impl Into<String>,
        descriptor: impl Into<String>,
        injection: Injection,
        helper: impl Into<String>,
    ) -> Self {
        Self {
            class_pattern: class_pattern.into(),
            method: method.into(),
            descriptor: descriptor.into(),
            injection,
            helper: helper.into(),
        }
    }
}

/// Compiled bytecode result: the modified class bytes.
pub struct TransformedClass {
    pub bytes: Vec<u8>,
}

/// The engine consumes class bytes, runs matching rules, returns new bytes.
pub struct TransformEngine {
    rules: Mutex<Vec<Arc<Rule>>>,
}

impl TransformEngine {
    pub fn new() -> Self {
        Self {
            rules: Mutex::new(Vec::new()),
        }
    }

    pub fn register(&self, rule: Rule) {
        self.rules.lock().unwrap().push(Arc::new(rule));
    }

    pub fn rules(&self) -> Vec<Arc<Rule>> {
        self.rules.lock().unwrap().clone()
    }

    /// Apply matching rules to class bytes.
    ///
    /// Returns `Ok(None)` if no rule matched, or if the class parsed but no
    /// edit was produced (e.g. all methods abstract, or no call site
    /// matched) — the class should be passed through untouched. If any rule
    /// matched but the transform failed, returns `Err` — the platform may
    /// then choose to fail the class load or run it untransformed. Never
    /// panics; parse failures return `Err` with context.
    pub fn apply(&self, class_name: &str, bytes: &[u8]) -> Result<Option<TransformedClass>, String> {
        let matched: Vec<Arc<Rule>> = self
            .rules()
            .into_iter()
            .filter(|r| matches_pattern(&r.class_pattern, class_name))
            .collect();
        if matched.is_empty() {
            return Ok(None);
        }
        let class = parse_class(bytes).map_err(|e| format!("transform {class_name}: {e}"))?;
        let mut plan = Plan::default();
        for rule in &matched {
            for m in class.methods.iter().filter(|m| rule_matches_method(rule, m)) {
                match &rule.injection {
                    Injection::MethodEntry => {
                        plan_method_entry(&class, m, &rule.helper, &mut plan)?;
                    }
                    Injection::BeforeCall(target) => {
                        plan_before_call(&class, m, target, &rule.helper, &mut plan)?;
                    }
                }
            }
        }
        if plan.is_empty() {
            return Ok(None);
        }
        let mut edits = plan.edits;
        if plan.cp_added > 0 {
            edits.push(Edit::Insert(class.cp_end, plan.cp_bytes.into_boxed_slice()));
            edits.push(Edit::SetU2(8, class.cp.len() as u16 + plan.cp_added));
        }
        // Set edits rewrite in place, so only Insert edits change payload
        // sizes. Every attribute whose payload grew (the Code attribute and
        // any StackMapTable inside it) must have its u4 attribute_length
        // field fixed up, in original-file offsets. Insert edits that merely
        // re-emit bytes consumed by a Set (StackMapTable u2 re-bumps) are
        // net-zero and excluded.
        let mut attr_len_edits: Vec<Edit> = Vec::new();
        for m in &class.methods {
            let Some(code) = &m.code else { continue };
            for (payload_off, payload_len) in
                std::iter::once((code.attr_len_off + 4, code.attr_len)).chain(code.sub.iter().map(|s| (s.off, s.len)))
            {
                let payload_end = payload_off + payload_len;
                let delta: i64 = edits
                    .iter()
                    .filter(|e| {
                        let o = e.offset();
                        o >= payload_off
                            && o <= payload_end
                            && !matches!(e, Edit::Insert(_, _) if plan.restores.contains(&o))
                    })
                    .map(|e| match e {
                        Edit::Insert(_, b) => b.len() as i64,
                        _ => 0,
                    })
                    .sum();
                if delta != 0 {
                    attr_len_edits.push(Edit::SetU4(
                        payload_off - 4,
                        (payload_len as i64 + delta) as u32,
                    ));
                }
            }
        }
        edits.extend(attr_len_edits);
        let out = apply_edits(bytes, edits)?;
        Ok(Some(TransformedClass { bytes: out }))
    }
}

fn matches_pattern(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*/") {
        return name.ends_with(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix);
    }
    pattern == name
}

fn rule_matches_method(rule: &Rule, m: &Member) -> bool {
    (rule.method == "*" || rule.method == m.name) && (rule.descriptor == "*" || rule.descriptor == m.descriptor)
}

/// Process-wide engine used by the platform hook pipeline.
static ENGINE: OnceLock<Arc<TransformEngine>> = OnceLock::new();

pub fn global_engine() -> Arc<TransformEngine> {
    ENGINE.get_or_init(|| Arc::new(TransformEngine::new())).clone()
}

// ---------------------------------------------------------------------------
// Class-file parser (JVMS chapter 4), std-only.
// ---------------------------------------------------------------------------

const MAGIC: u32 = 0xCAFE_BABE;
const TAG_UTF8: u8 = 1;
const TAG_INTEGER: u8 = 3;
const TAG_FLOAT: u8 = 4;
const TAG_LONG: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_CLASS: u8 = 7;
const TAG_STRING: u8 = 8;
const TAG_FIELDREF: u8 = 9;
const TAG_METHODREF: u8 = 10;
const TAG_IFACE_METHODREF: u8 = 11;
const TAG_NAME_AND_TYPE: u8 = 12;
const TAG_METHOD_HANDLE: u8 = 15;
const TAG_METHOD_TYPE: u8 = 16;
const TAG_DYNAMIC: u8 = 17;
const TAG_INVOKE_DYNAMIC: u8 = 18;
const TAG_MODULE: u8 = 19;
const TAG_PACKAGE: u8 = 20;

const OP_INVOKESTATIC: u8 = 0xB8;

/// One constant-pool entry: its tag plus the raw payload bytes following it.
struct CpEntry<'a> {
    tag: u8,
    payload: &'a [u8],
}

/// A parsed class file. `data` is the original buffer; everything else is a
/// structural view over it. Offsets in `CodeAttr`/`SubAttr` are file offsets
/// into `data`, valid only against the unmodified buffer.
struct ClassFile<'a> {
    data: &'a [u8],
    cp: Vec<CpEntry<'a>>,
    cp_end: usize,
    #[allow(dead_code)] // parsed for the transform contract; read by tests
    access_flags: u16,
    this_class: u16,
    #[allow(dead_code)]
    super_class: u16,
    #[allow(dead_code)]
    interfaces: Vec<u16>,
    #[allow(dead_code)]
    fields: Vec<Member>,
    methods: Vec<Member>,
}

struct Member {
    #[allow(dead_code)] // abstract/native is detected via absence of Code
    access_flags: u16,
    name: String,
    descriptor: String,
    code: Option<CodeAttr>,
}

/// The `Code` attribute of one method, with file offsets of the fields the
/// instrumenter must patch.
struct CodeAttr {
    /// File offset of the u4 `attribute_length` field (payload start - 4).
    attr_len_off: usize,
    /// Original `attribute_length` value (Code payload size in bytes).
    attr_len: usize,
    code_len_off: usize,
    code_off: usize,
    code_len: usize,
    /// File offsets of the `start_pc`/`end_pc`/`handler_pc` u2 fields of
    /// each exception-table entry (catch_type is never patched).
    exc: Vec<(usize, usize, usize)>,
    sub: Vec<SubAttr>,
}

/// An attribute located in the file: name, file offset of the
/// `attribute_name_index` u2, and `attribute_length`.
struct SubAttr {
    name: String,
    off: usize,
    len: usize,
}

/// Sequential reader with bounds checks.
struct Rd<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> Rd<'a> {
    fn u1(&mut self, ctx: &str) -> Result<u8, String> {
        let b = self
            .d
            .get(self.p)
            .copied()
            .ok_or_else(|| format!("{ctx}: truncated class file at offset {}", self.p))?;
        self.p += 1;
        Ok(b)
    }

    fn u2(&mut self, ctx: &str) -> Result<u16, String> {
        let b = self
            .d
            .get(self.p..self.p + 2)
            .ok_or_else(|| format!("{ctx}: truncated class file at offset {}", self.p))?;
        self.p += 2;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn u4(&mut self, ctx: &str) -> Result<u32, String> {
        let b = self
            .d
            .get(self.p..self.p + 4)
            .ok_or_else(|| format!("{ctx}: truncated class file at offset {}", self.p))?;
        self.p += 4;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn take(&mut self, n: usize, ctx: &str) -> Result<&'a [u8], String> {
        let b = self
            .d
            .get(self.p..self.p + n)
            .ok_or_else(|| format!("{ctx}: truncated class file at offset {}", self.p))?;
        self.p += n;
        Ok(b)
    }
}

fn parse_class(data: &[u8]) -> Result<ClassFile<'_>, String> {
    let mut r = Rd { d: data, p: 0 };
    let magic = r.u4("magic")?;
    if magic != MAGIC {
        return Err(format!("not a class file: bad magic {magic:#010x}"));
    }
    r.u2("minor_version")?;
    r.u2("major_version")?;
    let cp_count = r.u2("constant_pool_count")? as usize;
    if cp_count == 0 {
        return Err("constant_pool_count must be at least 1".to_string());
    }
    let mut cp: Vec<CpEntry<'_>> = Vec::with_capacity(cp_count);
    cp.push(CpEntry { tag: 0, payload: &[] });
    let mut i = 1usize;
    while i < cp_count {
        let tag = r.u1(&format!("constant pool #{i} tag"))?;
        let (payload, extra_slot) = match tag {
            TAG_UTF8 => {
                let l = r.u2(&format!("constant pool #{i} utf8 length"))? as usize;
                (r.take(l, &format!("constant pool #{i} utf8 bytes"))?, false)
            }
            TAG_INTEGER | TAG_FLOAT => (r.take(4, &format!("constant pool #{i}"))?, false),
            TAG_LONG | TAG_DOUBLE => (r.take(8, &format!("constant pool #{i}"))?, true),
            TAG_CLASS | TAG_STRING | TAG_METHOD_TYPE | TAG_MODULE | TAG_PACKAGE => {
                (r.take(2, &format!("constant pool #{i}"))?, false)
            }
            TAG_FIELDREF
            | TAG_METHODREF
            | TAG_IFACE_METHODREF
            | TAG_NAME_AND_TYPE
            | TAG_DYNAMIC
            | TAG_INVOKE_DYNAMIC => (r.take(4, &format!("constant pool #{i}"))?, false),
            TAG_METHOD_HANDLE => (r.take(3, &format!("constant pool #{i}"))?, false),
            t => return Err(format!("constant pool #{i}: unknown tag {t}")),
        };
        if extra_slot {
            // long/double occupy two slots; the second has no tag byte
            cp.push(CpEntry { tag: 0, payload: &[] });
            i += 1;
        }
        cp.push(CpEntry { tag, payload });
        i += 1;
    }
    let cp_end = r.p;
    let access_flags = r.u2("access_flags")?;
    let this_class = r.u2("this_class")?;
    let super_class = r.u2("super_class")?;
    let interfaces_count = r.u2("interfaces_count")? as usize;
    let mut interfaces = Vec::with_capacity(interfaces_count);
    for k in 0..interfaces_count {
        interfaces.push(r.u2(&format!("interface #{k}"))?);
    }
    let fields_count = r.u2("fields_count")? as usize;
    let mut fields = Vec::with_capacity(fields_count);
    for k in 0..fields_count {
        fields.push(parse_member(data, &cp, &mut r, &format!("field #{k}"))?);
    }
    let methods_count = r.u2("methods_count")? as usize;
    let mut methods = Vec::with_capacity(methods_count);
    for k in 0..methods_count {
        methods.push(parse_member(data, &cp, &mut r, &format!("method #{k}"))?);
    }
    let attrs_count = r.u2("class attributes_count")? as usize;
    for k in 0..attrs_count {
        parse_sub_attr(data, &cp, &mut r, &format!("class attribute #{k}"))?;
    }
    if r.p != data.len() {
        return Err(format!(
            "class file has {} trailing bytes after attributes",
            data.len() - r.p
        ));
    }
    Ok(ClassFile {
        data,
        cp,
        cp_end,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
    })
}

fn parse_member<'a>(
    data: &[u8],
    cp: &[CpEntry<'a>],
    r: &mut Rd<'a>,
    ctx: &str,
) -> Result<Member, String> {
    let access_flags = r.u2(&format!("{ctx} access_flags"))?;
    let name_idx = r.u2(&format!("{ctx} name_index"))?;
    let desc_idx = r.u2(&format!("{ctx} descriptor_index"))?;
    let name = utf8(cp, name_idx, ctx)?;
    let descriptor = utf8(cp, desc_idx, ctx)?;
    let attrs_count = r.u2(&format!("{ctx} attributes_count"))? as usize;
    let mut code = None;
    for k in 0..attrs_count {
        let a = parse_sub_attr(data, cp, r, &format!("{ctx} attribute #{k}"))?;
        if a.name == "Code" {
            if code.is_some() {
                return Err(format!("{ctx} has more than one Code attribute"));
            }
            code = Some(parse_code(data, cp, &a)?);
        }
    }
    Ok(Member {
        access_flags,
        name,
        descriptor,
        code,
    })
}

fn parse_sub_attr<'a>(
    _data: &[u8],
    cp: &[CpEntry<'a>],
    r: &mut Rd<'a>,
    ctx: &str,
) -> Result<SubAttr, String> {
    let name_idx = r.u2(&format!("{ctx} attribute_name_index"))?;
    let len = r.u4(&format!("{ctx} attribute_length"))? as usize;
    let name = utf8(cp, name_idx, ctx)?;
    let off = r.p;
    r.take(len, &format!("{ctx} ({name}) payload"))?;
    Ok(SubAttr { name, off, len })
}

fn parse_code(data: &[u8], cp: &[CpEntry<'_>], a: &SubAttr) -> Result<CodeAttr, String> {
    let end = a
        .off
        .checked_add(a.len)
        .filter(|&e| e <= data.len())
        .ok_or_else(|| "Code attribute extends past end of class file".to_string())?;
    let mut p = a.off;
    let code_len_off = p + 4;
    let code_len = u32_at(data, p + 4, "Code code_length")? as usize;
    if code_len == 0 {
        return Err("Code attribute with zero code_length".to_string());
    }
    if code_len > 0xFFFF {
        return Err("Code attribute code_length exceeds 65535".to_string());
    }
    let code_off = p + 8;
    if code_off + code_len > end {
        return Err("Code attribute code array extends past its payload".to_string());
    }
    p = code_off + code_len;
    let exc_count = u16_at(data, p, "Code exception_table_length")? as usize;
    p += 2;
    let mut exc = Vec::with_capacity(exc_count);
    for k in 0..exc_count {
        let start_off = p;
        let start_pc = u16_at(data, p, "exception start_pc")?;
        let end_off = p + 2;
        let end_pc = u16_at(data, p + 2, "exception end_pc")?;
        let handler_off = p + 4;
        let handler_pc = u16_at(data, p + 4, "exception handler_pc")?;
        let _catch_type = u16_at(data, p + 6, "exception catch_type")?;
        for (v, what) in [
            (start_pc, "start_pc"),
            (end_pc, "end_pc"),
            (handler_pc, "handler_pc"),
        ] {
            if v as usize > code_len {
                return Err(format!("exception table entry #{k}: {what} {v} beyond code_length {code_len}"));
            }
        }
        exc.push((start_off, end_off, handler_off));
        p += 8;
    }
    let sub_count = u16_at(data, p, "Code attributes_count")? as usize;
    p += 2;
    let mut sub = Vec::with_capacity(sub_count);
    for k in 0..sub_count {
        let mut sr = Rd { d: data, p };
        let s = parse_sub_attr(data, cp, &mut sr, &format!("Code attribute #{k}"))?;
        p = sr.p;
        sub.push(s);
    }
    if p != end {
        return Err(format!("Code attribute has {} trailing bytes", end - p));
    }
    Ok(CodeAttr {
        attr_len_off: a.off - 4,
        attr_len: a.len,
        code_len_off,
        code_off,
        code_len,
        exc,
        sub,
    })
}

fn utf8(cp: &[CpEntry<'_>], idx: u16, ctx: &str) -> Result<String, String> {
    let e = cp
        .get(idx as usize)
        .ok_or_else(|| format!("{ctx}: constant pool index {idx} out of range"))?;
    if e.tag != TAG_UTF8 {
        return Err(format!("{ctx}: constant pool #{idx} is not a Utf8 (tag {})", e.tag));
    }
    Ok(String::from_utf8_lossy(e.payload).into_owned())
}

/// Look up a constant-pool entry, returning `(tag, payload)`.
fn cp_get<'a>(class: &ClassFile<'a>, idx: u16) -> Result<(u8, &'a [u8]), String> {
    let e = class
        .cp
        .get(idx as usize)
        .ok_or_else(|| format!("constant pool index {idx} out of range"))?;
    Ok((e.tag, e.payload))
}

fn class_name(class: &ClassFile<'_>, idx: u16) -> Result<String, String> {
    let (tag, payload) = cp_get(class, idx)?;
    if tag != TAG_CLASS {
        return Err(format!("constant pool #{idx} is not a Class (tag {tag})"));
    }
    let name_idx = u16_at(payload, 0, "class name_index")?;
    utf8(&class.cp, name_idx, "class name")
}

fn name_and_type(class: &ClassFile<'_>, idx: u16) -> Result<(String, String), String> {
    let (tag, payload) = cp_get(class, idx)?;
    if tag != TAG_NAME_AND_TYPE {
        return Err(format!("constant pool #{idx} is not a NameAndType (tag {tag})"));
    }
    let name_idx = u16_at(payload, 0, "NameAndType name_index")?;
    let desc_idx = u16_at(payload, 2, "NameAndType descriptor_index")?;
    Ok((utf8(&class.cp, name_idx, "NameAndType name")?, utf8(&class.cp, desc_idx, "NameAndType descriptor")?))
}

/// Resolve a Methodref/InterfaceMethodref to `(owner, name, descriptor)`.
fn resolve_methodref(class: &ClassFile<'_>, idx: u16) -> Result<(String, String, String), String> {
    let (tag, payload) = cp_get(class, idx)?;
    if !matches!(tag, TAG_METHODREF | TAG_IFACE_METHODREF) {
        return Err(format!("constant pool #{idx} is not a method reference (tag {tag})"));
    }
    let class_idx = u16_at(payload, 0, "methodref class_index")?;
    let nat_idx = u16_at(payload, 2, "methodref name_and_type_index")?;
    let owner = class_name(class, class_idx)?;
    let (name, descriptor) = name_and_type(class, nat_idx)?;
    Ok((owner, name, descriptor))
}

fn u16_at(d: &[u8], off: usize, ctx: &str) -> Result<u16, String> {
    let b = d
        .get(off..off + 2)
        .ok_or_else(|| format!("{ctx}: truncated read at offset {off}"))?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}

fn u32_at(d: &[u8], off: usize, ctx: &str) -> Result<u32, String> {
    let b = d
        .get(off..off + 4)
        .ok_or_else(|| format!("{ctx}: truncated read at offset {off}"))?;
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn s16_at(d: &[u8], off: usize, ctx: &str) -> Result<i32, String> {
    Ok(u16_at(d, off, ctx)? as i16 as i32)
}

fn s32_at(d: &[u8], off: usize, ctx: &str) -> Result<i32, String> {
    Ok(u32_at(d, off, ctx)? as i32)
}

// ---------------------------------------------------------------------------
// Instrumentation planning and surgical edits.
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Edit {
    SetU1(usize, u8),
    SetU2(usize, u16),
    SetU4(usize, u32),
    Insert(usize, Box<[u8]>),
}

impl Edit {
    fn offset(&self) -> usize {
        match *self {
            Edit::SetU1(o, _) | Edit::SetU2(o, _) | Edit::SetU4(o, _) | Edit::Insert(o, _) => o,
        }
    }
}

/// Accumulates everything needed to rebuild a transformed class.
#[derive(Default)]
struct Plan {
    edits: Vec<Edit>,
    /// Serialized constant-pool entries appended after the original pool.
    cp_bytes: Vec<u8>,
    /// Number of entries appended (long/double would count 2; ours never do).
    cp_added: u16,
    cp_utf8: HashMap<String, u16>,
    cp_class: HashMap<u16, u16>,
    cp_nat: HashMap<(u16, u16), u16>,
    cp_mref: HashMap<(u16, u16), u16>,
    helpers: HashMap<String, u16>,
    /// Cumulative signed fixup delta per file offset, so several insertions
    /// that shift the same field sum up instead of overwriting (the last
    /// Set edit written carries the full cumulative value).
    sums: HashMap<usize, i64>,
    /// StackMapTable frames already bumped, keyed by the file offset of
    /// their frame_type byte; true once the frame has been rewritten in
    /// extended (u2 delta) form so the conversion is emitted only once.
    smap: HashMap<usize, bool>,
    /// File offsets of Insert edits that merely re-emit bytes consumed by a
    /// Set edit (StackMapTable u2 re-bumps); excluded from attribute-length
    /// deltas since they are net-zero.
    restores: Vec<usize>,
}

impl Plan {
    fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    /// Set a u2 field at `off` to `original + delta`, accumulating `delta`
    /// per offset so repeated shifts compose.
    fn bump_u2(&mut self, off: usize, original: u16, delta: i64) {
        let cum = self.sums.entry(off).or_insert(0);
        *cum += delta;
        self.edits.push(Edit::SetU2(off, (i64::from(original) + *cum) as u16));
    }

    /// Set a u4 field at `off` to `original + delta`, accumulating `delta`
    /// per offset so repeated shifts compose.
    fn bump_u4(&mut self, off: usize, original: u32, delta: i64) {
        let cum = self.sums.entry(off).or_insert(0);
        *cum += delta;
        self.edits.push(Edit::SetU4(off, (i64::from(original) + *cum) as u32));
    }
}

/// Index of the next constant-pool entry to append (1-based, per JVMS).
/// `cp.len()` already counts the reserved index-0 slot, so it equals
/// `constant_pool_count` — the next free index is exactly that.
fn cp_index(class: &ClassFile<'_>, plan: &Plan) -> Result<u16, String> {
    let base = class.cp.len() as u32;
    let idx = base + u32::from(plan.cp_added);
    if idx > u32::from(u16::MAX) {
        return Err("constant pool exhausted (>= 65535 entries)".to_string());
    }
    Ok(idx as u16)
}

fn cp_utf8(class: &ClassFile<'_>, plan: &mut Plan, s: &str) -> Result<u16, String> {
    if let Some(i) = class.cp.iter().enumerate().skip(1).find_map(|(i, e)| {
        (e.tag == TAG_UTF8 && e.payload == s.as_bytes()).then_some(i as u16)
    }) {
        return Ok(i);
    }
    if let Some(&i) = plan.cp_utf8.get(s) {
        return Ok(i);
    }
    let idx = cp_index(class, plan)?;
    if s.len() > usize::from(u16::MAX) {
        return Err("Utf8 constant too long".to_string());
    }
    plan.cp_utf8.insert(s.to_string(), idx);
    plan.cp_bytes.push(TAG_UTF8);
    plan.cp_bytes.extend_from_slice(&(s.len() as u16).to_be_bytes());
    plan.cp_bytes.extend_from_slice(s.as_bytes());
    plan.cp_added += 1;
    Ok(idx)
}

fn cp_class(class: &ClassFile<'_>, plan: &mut Plan, name_utf8: u16) -> Result<u16, String> {
    if let Some(i) = class.cp.iter().enumerate().skip(1).find_map(|(i, e)| {
        (e.tag == TAG_CLASS && e.payload.get(..2).map(|p| u16::from_be_bytes([p[0], p[1]])) == Some(name_utf8))
            .then_some(i as u16)
    }) {
        return Ok(i);
    }
    if let Some(&i) = plan.cp_class.get(&name_utf8) {
        return Ok(i);
    }
    let idx = cp_index(class, plan)?;
    plan.cp_class.insert(name_utf8, idx);
    plan.cp_bytes.push(TAG_CLASS);
    plan.cp_bytes.extend_from_slice(&name_utf8.to_be_bytes());
    plan.cp_added += 1;
    Ok(idx)
}

fn cp_name_and_type(class: &ClassFile<'_>, plan: &mut Plan, name: u16, desc: u16) -> Result<u16, String> {
    let existing = class.cp.iter().enumerate().skip(1).find_map(|(i, e)| {
        (e.tag == TAG_NAME_AND_TYPE
            && e.payload.get(..4).map(|p| {
                let n = u16::from_be_bytes([p[0], p[1]]);
                let d = u16::from_be_bytes([p[2], p[3]]);
                (n, d)
            }) == Some((name, desc)))
        .then_some(i as u16)
    });
    if let Some(i) = existing {
        return Ok(i);
    }
    if let Some(&i) = plan.cp_nat.get(&(name, desc)) {
        return Ok(i);
    }
    let idx = cp_index(class, plan)?;
    plan.cp_nat.insert((name, desc), idx);
    plan.cp_bytes.push(TAG_NAME_AND_TYPE);
    plan.cp_bytes.extend_from_slice(&name.to_be_bytes());
    plan.cp_bytes.extend_from_slice(&desc.to_be_bytes());
    plan.cp_added += 1;
    Ok(idx)
}

fn cp_methodref(class: &ClassFile<'_>, plan: &mut Plan, class_idx: u16, nat_idx: u16) -> Result<u16, String> {
    let existing = class.cp.iter().enumerate().skip(1).find_map(|(i, e)| {
        (e.tag == TAG_METHODREF
            && e.payload.get(..4).map(|p| {
                let c = u16::from_be_bytes([p[0], p[1]]);
                let n = u16::from_be_bytes([p[2], p[3]]);
                (c, n)
            }) == Some((class_idx, nat_idx)))
        .then_some(i as u16)
    });
    if let Some(i) = existing {
        return Ok(i);
    }
    if let Some(&i) = plan.cp_mref.get(&(class_idx, nat_idx)) {
        return Ok(i);
    }
    let idx = cp_index(class, plan)?;
    plan.cp_mref.insert((class_idx, nat_idx), idx);
    plan.cp_bytes.push(TAG_METHODREF);
    plan.cp_bytes.extend_from_slice(&class_idx.to_be_bytes());
    plan.cp_bytes.extend_from_slice(&nat_idx.to_be_bytes());
    plan.cp_added += 1;
    Ok(idx)
}

/// Resolve (appending if needed) the `invokestatic` target for `helper`,
/// returning its constant-pool index. The helper must be a static method
/// with descriptor `()V`.
fn helper_methodref(class: &ClassFile<'_>, plan: &mut Plan, helper: &str) -> Result<u16, String> {
    if let Some(&i) = plan.helpers.get(helper) {
        return Ok(i);
    }
    let (owner, name) = helper
        .rsplit_once('.')
        .ok_or_else(|| format!("helper must be 'Owner.Class.method', got '{helper}'"))?;
    if owner.is_empty() || name.is_empty() {
        return Err(format!("helper must be 'Owner.Class.method', got '{helper}'"));
    }
    let owner = owner.replace('.', "/");
    let c = cp_utf8(class, plan, &owner)?;
    let n = cp_utf8(class, plan, name)?;
    let d = cp_utf8(class, plan, "()V")?;
    let cc = cp_class(class, plan, c)?;
    let nt = cp_name_and_type(class, plan, n, d)?;
    let mr = cp_methodref(class, plan, cc, nt)?;
    plan.helpers.insert(helper.to_string(), mr);
    Ok(mr)
}

fn invokestatic_bytes(mr: u16) -> [u8; 3] {
    [OP_INVOKESTATIC, (mr >> 8) as u8, mr as u8]
}

/// True if the instruction at code-relative `pc` is `invokestatic` to the
/// constant-pool entry `mr` (idempotency check for retransformation).
fn code_at_invokes_helper(class: &ClassFile<'_>, code: &CodeAttr, pc: usize, mr: u16) -> bool {
    code.code_len >= pc + 3
        && class.data[code.code_off + pc] == OP_INVOKESTATIC
        && class.data[code.code_off + pc + 1] as u16 * 256 + class.data[code.code_off + pc + 2] as u16 == mr
}

fn plan_method_entry(class: &ClassFile<'_>, m: &Member, helper: &str, plan: &mut Plan) -> Result<(), String> {
    let code = match &m.code {
        Some(c) => c,
        // abstract/native methods have no Code attribute — nothing to inject
        None => return Ok(()),
    };
    let mr = helper_methodref(class, plan, helper)?;
    if code_at_invokes_helper(class, code, 0, mr) {
        return Ok(());
    }
    plan.edits.push(Edit::Insert(code.code_off, invokestatic_bytes(mr).into()));
    plan.bump_u4(code.code_len_off, code.code_len as u32, 3);
    plan_code_fixups(class, code, 0, 3, plan)
}

fn plan_before_call(
    class: &ClassFile<'_>,
    m: &Member,
    target: &str,
    helper: &str,
    plan: &mut Plan,
) -> Result<(), String> {
    let code = match &m.code {
        Some(c) => c,
        None => return Ok(()),
    };
    let mr = helper_methodref(class, plan, helper)?;
    let code_bytes = &class.data[code.code_off..code.code_off + code.code_len];
    let instrs = instr_sizes(code_bytes)?;
    for &(o, _, op) in &instrs {
        if !(0xB6..=0xB9).contains(&op) {
            continue;
        }
        let idx = u16_at(code_bytes, o + 1, "invoke methodref index")?;
        let resolved = resolve_methodref(class, idx)?;
        if !target_matches(target, &resolved) {
            continue;
        }
        if o >= 3 && code_at_invokes_helper(class, code, o - 3, mr) {
            continue;
        }
        plan.edits.push(Edit::Insert(code.code_off + o, invokestatic_bytes(mr).into()));
        plan.bump_u4(code.code_len_off, code.code_len as u32, 3);
        plan_code_fixups(class, code, o, 3, plan)?;
    }
    Ok(())
}

fn target_matches(target: &str, resolved: &(String, String, String)) -> bool {
    let (owner, name, desc) = resolved;
    let full = format!("{owner}.{name}:{desc}");
    if target == full {
        return true;
    }
    // owner-agnostic form "name:desc" — the name part (before ':') must not
    // contain a '/', since an owner-qualified target always does; descriptor
    // slashes (e.g. Ljava/lang/String) live after the ':'.
    let Some((name_part, desc_part)) = target.split_once(':') else {
        return false;
    };
    !name_part.contains('/') && name_part == name && desc_part == desc
}

/// Delta to add to a branch operand after inserting `ins_len` bytes at
/// `pc`: only branches that span the insertion point change, because both
/// the branch and its target keep their side relative to the shift. A target
/// exactly at `pc` stays put (it now points at the hook, preserving the
/// "hook first" semantics).
fn branch_delta(o: usize, t: i32, pc: usize, ins_len: i32) -> i32 {
    let pc = pc as i32;
    if t == pc {
        0
    } else if (o as i32) < pc && t > pc {
        ins_len
    } else if (o as i32) >= pc && t < pc {
        -ins_len
    } else {
        0
    }
}

/// Instruction sizes, per JVMS chapter 6: (offset, size, opcode).
fn instr_sizes(code: &[u8]) -> Result<Vec<(usize, usize, u8)>, String> {
    let mut out = Vec::new();
    let mut p = 0usize;
    while p < code.len() {
        let op = code[p];
        let len = instr_len(code, p, op)?;
        out.push((p, len, op));
        p += len;
    }
    Ok(out)
}

fn instr_len(code: &[u8], p: usize, op: u8) -> Result<usize, String> {
    let rem = code.len() - p;
    let need = |n: usize| -> Result<(), String> {
        if rem < n {
            Err(format!("truncated instruction at offset {p} (opcode {op:#04x})"))
        } else {
            Ok(())
        }
    };
    let fixed = |n: usize| -> Result<usize, String> {
        need(n)?;
        Ok(n)
    };
    match op {
        0x00..=0x0F => Ok(1),
        0x10 => fixed(2),
        0x11 => fixed(3),
        0x12 => fixed(2),
        0x13..=0x14 => fixed(3),
        0x15..=0x19 => fixed(2),
        0x1A..=0x35 => Ok(1),
        0x36..=0x3A => fixed(2),
        0x3B..=0x83 => Ok(1),
        0x84 => fixed(3),
        0x85..=0x98 => Ok(1),
        0x99..=0xA8 => fixed(3),
        0xA9 => fixed(2),
        0xAA => {
            need(1)?;
            let pad = (4 - ((p + 1) % 4)) % 4;
            let ops = p + 1 + pad;
            need(12)?;
            let high = s32_at(code, ops + 8, "tableswitch high")?;
            let low = s32_at(code, ops + 4, "tableswitch low")?;
            if high < low {
                return Err(format!("tableswitch at offset {p}: high < low"));
            }
            let n = i64::from(high) - i64::from(low) + 1;
            let total = 1usize + pad + 12 + n as usize * 4;
            need(total)?;
            Ok(total)
        }
        0xAB => {
            need(1)?;
            let pad = (4 - ((p + 1) % 4)) % 4;
            let ops = p + 1 + pad;
            need(8)?;
            let npairs = s32_at(code, ops + 4, "lookupswitch npairs")?;
            if npairs < 0 {
                return Err(format!("lookupswitch at offset {p}: negative npairs"));
            }
            let total = 1usize + pad + 8 + npairs as usize * 8;
            need(total)?;
            Ok(total)
        }
        0xAC..=0xB1 => Ok(1),
        0xB2..=0xB8 => fixed(3),
        0xB9..=0xBA => fixed(5),
        0xBB => fixed(3),
        0xBC => fixed(2),
        0xBD => fixed(3),
        0xBE..=0xBF => Ok(1),
        0xC0..=0xC1 => fixed(3),
        0xC2..=0xC3 => Ok(1),
        0xC4 => {
            need(2)?;
            match code[p + 1] {
                0x15..=0x19 | 0x36..=0x3A | 0xA9 => fixed(4),
                0x84 => fixed(6),
                _ => Err(format!("wide with invalid opcode {:#04x} at offset {p}", code[p + 1])),
            }
        }
        0xC5 => fixed(4),
        0xC6..=0xC7 => fixed(3),
        0xC8..=0xC9 => fixed(5),
        // breakpoint, reserved, impdep — treat as opaque 1-byte opcodes
        0xCA..=0xFF => Ok(1),
    }
}

/// Rewrite branch operands that span the insertion point.
fn plan_branch_fixups(class: &ClassFile<'_>, code: &CodeAttr, pc: usize, ins_len: i32, plan: &mut Plan) -> Result<(), String> {
    let code_bytes = &class.data[code.code_off..code.code_off + code.code_len];
    let instrs = instr_sizes(code_bytes)?;
    for &(o, _, op) in &instrs {
        match op {
            0x99..=0xA6 | 0xA7 | 0xA8 | 0xC6 | 0xC7 => {
                let pos = o + 1;
                let t = o as i32 + s16_at(code_bytes, pos, "branch operand")?;
                let d = branch_delta(o, t, pc, ins_len);
                if d != 0 {
                    let operand = u16_at(code_bytes, pos, "branch operand")?;
                    plan.bump_u2(code.code_off + pos, operand, d.into());
                }
            }
            0xC8 | 0xC9 => {
                let pos = o + 1;
                let t = o as i32 + s32_at(code_bytes, pos, "wide branch operand")?;
                let d = branch_delta(o, t, pc, ins_len);
                if d != 0 {
                    let operand = u32_at(code_bytes, pos, "wide branch operand")?;
                    plan.bump_u4(code.code_off + pos, operand, d.into());
                }
            }
            0xAA => {
                let pad = (4 - ((o + 1) % 4)) % 4;
                let ops = o + 1 + pad;
                let high = s32_at(code_bytes, ops + 8, "tableswitch high")?;
                let low = s32_at(code_bytes, ops + 4, "tableswitch low")?;
                let targets = (0..=high - low).map(|k| ops + 12 + k as usize * 4).chain([ops]);
                for pos in targets {
                    let t = o as i32 + s32_at(code_bytes, pos, "tableswitch target")?;
                    let d = branch_delta(o, t, pc, ins_len);
                    if d != 0 {
                        let operand = u32_at(code_bytes, pos, "tableswitch target")?;
                        plan.bump_u4(code.code_off + pos, operand, d.into());
                    }
                }
            }
            0xAB => {
                let pad = (4 - ((o + 1) % 4)) % 4;
                let ops = o + 1 + pad;
                let npairs = s32_at(code_bytes, ops + 4, "lookupswitch npairs")?;
                for k in 0..npairs {
                    let pos = ops + 8 + k as usize * 8 + 4;
                    let t = o as i32 + s32_at(code_bytes, pos, "lookupswitch target")?;
                    let d = branch_delta(o, t, pc, ins_len);
                    if d != 0 {
                        let operand = u32_at(code_bytes, pos, "lookupswitch target")?;
                        plan.bump_u4(code.code_off + pos, operand, d.into());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Patch all Code-internal structures whose offsets shift when `ins_len`
/// bytes are inserted at code-relative position `pc`.
fn plan_code_fixups(class: &ClassFile<'_>, code: &CodeAttr, pc: usize, ins_len: i32, plan: &mut Plan) -> Result<(), String> {
    plan_branch_fixups(class, code, pc, ins_len, plan)?;
    for &(start_off, end_off, handler_off) in &code.exc {
        let s = u16_at(class.data, start_off, "exception start_pc")?;
        if s as usize > pc {
            plan.bump_u2(start_off, s, ins_len.into());
        }
        let e = u16_at(class.data, end_off, "exception end_pc")?;
        // end_pc is exclusive: an entry ending at the old code end or past
        // the insertion point shifts with it
        if e as usize >= code.code_len || e as usize > pc {
            plan.bump_u2(end_off, e, ins_len.into());
        }
        let h = u16_at(class.data, handler_off, "exception handler_pc")?;
        if h as usize > pc {
            plan.bump_u2(handler_off, h, ins_len.into());
        }
    }
    for sub in &code.sub {
        match sub.name.as_str() {
            "StackMapTable" => plan_stackmap(class, sub, pc, ins_len, plan)?,
                    "LineNumberTable" => {
                        let payload = class.data.get(sub.off..sub.off + sub.len).ok_or_else(|| {
                            "LineNumberTable attribute extends past end of class file".to_string()
                        })?;
                let count = u16_at(payload, 0, "LineNumberTable number_of_entries")? as usize;
                let mut p = 2usize;
                for _ in 0..count {
                    let start = u16_at(payload, p, "LineNumberTable start_pc")?;
                    if start as usize > pc {
                        plan.bump_u2(sub.off + p, start, ins_len.into());
                    }
                    p += 4;
                }
            }
            "LocalVariableTable" | "LocalVariableTypeTable" => {
                let payload = class.data.get(sub.off..sub.off + sub.len).ok_or_else(|| {
                    format!("{} attribute extends past end of class file", sub.name)
                })?;
                let count = u16_at(payload, 0, "local variable table number_of_entries")? as usize;
                let mut p = 2usize;
                for _ in 0..count {
                    let start = u16_at(payload, p, "local variable start_pc")?;
                    let length = u16_at(payload, p + 2, "local variable length")?;
                    if start as usize > pc {
                        plan.bump_u2(sub.off + p, start, ins_len.into());
                    } else if start as usize + length as usize > pc {
                        plan.bump_u2(sub.off + p + 2, length, ins_len.into());
                    }
                    p += 10;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Size of a verification_type_info at `off` (bounds-checked against `end`).
fn vti_size(d: &[u8], off: usize, end: usize, ctx: &str) -> Result<usize, String> {
    let tag = d
        .get(off)
        .copied()
        .ok_or_else(|| format!("{ctx}: truncated verification_type_info"))?;
    match tag {
        0..=6 => Ok(1),
        7 | 8 => {
            if off + 3 > end {
                Err(format!("{ctx}: truncated verification_type_info (object/uninitialized)"))
            } else {
                Ok(3)
            }
        }
        t => Err(format!("{ctx}: unknown verification_type_info tag {t}")),
    }
}

/// StackMapTable frames are delta-encoded (JVMS 4.7.4): only the first frame
/// past the insertion point changes — its delta grows by `ins_len` — and the
/// encoding is re-chosen when the inline 6-bit delta overflows.
fn plan_stackmap(class: &ClassFile<'_>, sub: &SubAttr, pc: usize, ins_len: i32, plan: &mut Plan) -> Result<(), String> {
    let payload = class
        .data
        .get(sub.off..sub.off + sub.len)
        .ok_or_else(|| "StackMapTable attribute extends past end of class file".to_string())?;
    let end = payload.len();
    let count = u16_at(payload, 0, "StackMapTable number_of_entries")? as usize;
    let mut pos = 2usize;
    let mut prev_abs: i64 = -1;
    let mut bumped = false;
    for k in 0..count {
        let ft = payload
            .get(pos)
            .copied()
            .ok_or_else(|| format!("StackMapTable entry #{k}: truncated frame_type"))?;
        let ctx = |what: &str| format!("StackMapTable entry #{k} ({what})");
        let (delta, entry_len): (u16, usize) = match ft {
            0..=63 => (u16::from(ft), 1),
            64..=127 => (u16::from(ft - 64), 1 + vti_size(payload, pos + 1, end, &ctx("same_locals_1_stack_item"))?),
            247 => (
                u16_at(payload, pos + 1, &ctx("offset_delta"))?,
                3 + vti_size(payload, pos + 3, end, &ctx("same_locals_1_stack_item_extended"))?,
            ),
            248..=250 => (u16_at(payload, pos + 1, &ctx("offset_delta"))?, 3),
            251 => (u16_at(payload, pos + 1, &ctx("offset_delta"))?, 3),
            252..=254 => {
                let n = usize::from(ft - 251);
                let mut size = 3usize;
                let mut p = pos + 3;
                for _ in 0..n {
                    size += vti_size(payload, p, end, &ctx("append_frame local"))?;
                    p += 3;
                }
                (u16_at(payload, pos + 1, &ctx("offset_delta"))?, size)
            }
            255 => {
                let locals = u16_at(payload, pos + 3, &ctx("number_of_locals"))? as usize;
                let stack = u16_at(payload, pos + 5, &ctx("number_of_stack_items"))? as usize;
                let mut size = 7usize;
                let mut p = pos + 7;
                for _ in 0..locals {
                    size += vti_size(payload, p, end, &ctx("full_frame local"))?;
                    p += 3;
                }
                for _ in 0..stack {
                    size += vti_size(payload, p, end, &ctx("full_frame stack item"))?;
                    p += 3;
                }
                (u16_at(payload, pos + 1, &ctx("offset_delta"))?, size)
            }
            128..=246 => return Err(ctx("reserved frame_type")),
        };
        if pos + entry_len > end {
            return Err(ctx("entry extends past end of attribute"));
        }
        let abs = if k == 0 { i64::from(delta) } else { prev_abs + i64::from(delta) + 1 };
        prev_abs = abs;
        if !bumped && abs > pc as i64 {
            let key = sub.off + pos;
            let nd = i64::from(delta) + i64::from(ins_len);
            if nd > i64::from(u16::MAX) {
                return Err(ctx("offset_delta overflows u16"));
            }
            let nd = nd as u16;
            if ft <= 127 {
                match plan.smap.get(&key).copied() {
                    // first bump: re-encode the inline frame (keep inline, or
                    // grow to the extended form 251/247 + u2 delta)
                    None if nd <= 63 => {
                        plan.edits.push(Edit::SetU1(sub.off + pos, if ft <= 63 { nd as u8 } else { 64 + nd as u8 }));
                        plan.smap.insert(key, false);
                    }
                    None => {
                        plan.edits.push(Edit::SetU1(sub.off + pos, if ft <= 63 { 251 } else { 247 }));
                        plan.edits.push(Edit::Insert(sub.off + pos + 1, nd.to_be_bytes().into()));
                        plan.smap.insert(key, true);
                    }
                    // still inline: rewrite the frame_type
                    Some(false) if nd <= 63 => {
                        plan.edits.push(Edit::SetU1(sub.off + pos, if ft <= 63 { nd as u8 } else { 64 + nd as u8 }));
                    }
                    // inline, now overflows: convert to extended form
                    Some(false) => {
                        plan.edits.push(Edit::SetU1(sub.off + pos, if ft <= 63 { 251 } else { 247 }));
                        plan.edits.push(Edit::Insert(sub.off + pos + 1, nd.to_be_bytes().into()));
                        plan.smap.insert(key, true);
                    }
                    // already extended: rewrite the u2 delta and re-emit the
                    // two original bytes it consumes (they follow the u2 in
                    // the output)
                    Some(true) => {
                        let restore_off = sub.off + pos + 3;
                        plan.bump_u2(sub.off + pos + 1, delta, ins_len.into());
                        if restore_off <= class.data.len() {
                            plan.edits.push(Edit::Insert(
                                restore_off,
                                class.data[sub.off + pos + 1..sub.off + pos + 3].to_vec().into(),
                            ));
                            plan.restores.push(restore_off);
                        }
                    }
                }
            } else {
                // extended frame types (248..=255) carry a u2 delta
                plan.bump_u2(sub.off + pos + 1, delta, ins_len.into());
            }
            bumped = true;
        }
        pos += entry_len;
    }
    Ok(())
}

/// Rebuild the class bytes from the original buffer plus edits. Edits are
/// computed against the original layout; insertions at the same offset
/// append in order, a single Set may accompany them (a conflict is an
/// error).
fn apply_edits(data: &[u8], mut edits: Vec<Edit>) -> Result<Vec<u8>, String> {
    edits.sort_by_key(|e| e.offset());
    let mut out: Vec<u8> = Vec::with_capacity(data.len() + 64);
    let mut pos = 0usize;
    let mut i = 0usize;
    while i < edits.len() {
        let off = edits[i].offset();
        if off > data.len() {
            return Err(format!("edit at offset {off} beyond end of class file"));
        }
        out.extend_from_slice(&data[pos..off]);
        let mut chunk: Vec<u8> = Vec::new();
        let mut replaced = 0usize;
        while i < edits.len() && edits[i].offset() == off {
            match edits[i] {
                Edit::Insert(_, ref b) => chunk.extend_from_slice(b),
                // A Set describes the final bytes at this offset: the last
                // one wins, even over co-located Insert edits (e.g. a
                // StackMapTable frame converted to extended form whose u2
                // delta is re-bumped by a later insertion).
                Edit::SetU1(_, v) => {
                    chunk.clear();
                    chunk.push(v);
                    replaced = 1;
                }
                Edit::SetU2(_, v) => {
                    chunk.clear();
                    chunk.extend_from_slice(&v.to_be_bytes());
                    replaced = 2;
                }
                Edit::SetU4(_, v) => {
                    chunk.clear();
                    chunk.extend_from_slice(&v.to_be_bytes());
                    replaced = 4;
                }
            }
            i += 1;
        }
        out.extend_from_slice(&chunk);
        pos = off + replaced;
    }
    out.extend_from_slice(&data[pos..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- synthetic class builder -------------------------------------------

    struct Cb {
        cp: Vec<(u8, Vec<u8>)>,
    }

    impl Cb {
        fn new() -> Self {
            Self { cp: vec![(0, vec![])] }
        }
        fn utf8(&mut self, s: &str) -> u16 {
            let n = self.cp.len() as u16;
            let mut payload = (s.len() as u16).to_be_bytes().to_vec();
            payload.extend_from_slice(s.as_bytes());
            self.cp.push((TAG_UTF8, payload));
            n
        }
        fn class(&mut self, name_utf8: u16) -> u16 {
            let n = self.cp.len() as u16;
            self.cp.push((TAG_CLASS, name_utf8.to_be_bytes().to_vec()));
            n
        }
        fn nat(&mut self, name: u16, desc: u16) -> u16 {
            let n = self.cp.len() as u16;
            let mut p = name.to_be_bytes().to_vec();
            p.extend_from_slice(&desc.to_be_bytes());
            self.cp.push((TAG_NAME_AND_TYPE, p));
            n
        }
        fn mref(&mut self, class: u16, nat: u16) -> u16 {
            let n = self.cp.len() as u16;
            let mut p = class.to_be_bytes().to_vec();
            p.extend_from_slice(&nat.to_be_bytes());
            self.cp.push((TAG_METHODREF, p));
            n
        }
        fn long(&mut self, value: i64) -> u16 {
            let n = self.cp.len() as u16;
            self.cp.push((TAG_LONG, value.to_be_bytes().to_vec()));
            // long/double occupy two constant-pool slots; the second slot has
            // no tag byte in the file (fin filters tag-0 entries)
            self.cp.push((0, Vec::new()));
            n
        }
        fn idx(&self, s: &str) -> u16 {
            self.cp
                .iter()
                .enumerate()
                .find_map(|(i, (t, p))| (t == &TAG_UTF8 && p[2..] == *s.as_bytes()).then_some(i as u16))
                .expect("utf8 entry")
        }
        fn fin(&self, this: u16, super_: u16, methods: Vec<MethodB>) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(&0xCAFE_BABEu32.to_be_bytes());
            out.extend_from_slice(&0u16.to_be_bytes());
            out.extend_from_slice(&52u16.to_be_bytes());
            out.extend_from_slice(&(self.cp.len() as u16).to_be_bytes());
            for (tag, payload) in self.cp.iter().skip(1).filter(|(t, _)| *t != 0) {
                out.push(*tag);
                out.extend_from_slice(payload);
            }
            out.extend_from_slice(&0x0021u16.to_be_bytes());
            out.extend_from_slice(&this.to_be_bytes());
            out.extend_from_slice(&super_.to_be_bytes());
            out.extend_from_slice(&0u16.to_be_bytes());
            out.extend_from_slice(&0u16.to_be_bytes());
            out.extend_from_slice(&(methods.len() as u16).to_be_bytes());
            for m in methods {
                out.extend_from_slice(&m.access.to_be_bytes());
                out.extend_from_slice(&m.name.to_be_bytes());
                out.extend_from_slice(&m.desc.to_be_bytes());
                if let Some(c) = m.code {
                    let attr_len = 8
                        + c.code.len()
                        + 2
                        + c.exc.len() * 8
                        + 2
                        + c.sub.iter().map(|(_, p)| 6 + p.len()).sum::<usize>();
                    out.extend_from_slice(&1u16.to_be_bytes());
                    out.extend_from_slice(&c.code_attr_name.to_be_bytes());
                    out.extend_from_slice(&(attr_len as u32).to_be_bytes());
                    out.extend_from_slice(&c.max_stack.to_be_bytes());
                    out.extend_from_slice(&c.max_locals.to_be_bytes());
                    out.extend_from_slice(&(c.code.len() as u32).to_be_bytes());
                    out.extend_from_slice(&c.code);
                    out.extend_from_slice(&(c.exc.len() as u16).to_be_bytes());
                    for e in c.exc {
                        for v in e {
                            out.extend_from_slice(&v.to_be_bytes());
                        }
                    }
                    out.extend_from_slice(&(c.sub.len() as u16).to_be_bytes());
                    for (n, p) in c.sub {
                        out.extend_from_slice(&n.to_be_bytes());
                        out.extend_from_slice(&(p.len() as u32).to_be_bytes());
                        out.extend_from_slice(&p);
                    }
                } else {
                    out.extend_from_slice(&0u16.to_be_bytes());
                }
            }
            out.extend_from_slice(&0u16.to_be_bytes());
            out
        }
    }

    struct MethodB {
        access: u16,
        name: u16,
        desc: u16,
        code: Option<CodeB>,
    }

    struct CodeB {
        code_attr_name: u16,
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exc: Vec<[u16; 4]>,
        sub: Vec<(u16, Vec<u8>)>,
    }

    fn basic_cb() -> (Cb, u16, u16) {
        let mut c = Cb::new();
        let t = c.utf8("Test");
        let o = c.utf8("java/lang/Object");
        let cl_t = c.class(t);
        let cl_o = c.class(o);
        (c, cl_t, cl_o)
    }

    fn method_b(name: u16, desc: u16, code: Option<CodeB>) -> MethodB {
        MethodB {
            access: 0x0009,
            name,
            desc,
            code,
        }
    }

    fn code_b(code_attr_name: u16, code: Vec<u8>) -> CodeB {
        CodeB {
            code_attr_name,
            max_stack: 1,
            max_locals: 1,
            code,
            exc: vec![],
            sub: vec![],
        }
    }

    fn hook() -> Rule {
        Rule::new("Test", "run", "()V", Injection::MethodEntry, "dev.crussty.hooks.TickHook.onEntry")
    }

    fn code_of<'a>(class: &'a ClassFile<'a>, i: usize) -> &'a CodeAttr {
        class.methods[i].code.as_ref().expect("method has Code")
    }

    fn code_bytes<'a>(class: &'a ClassFile<'a>, i: usize) -> &'a [u8] {
        let c = code_of(class, i);
        &class.data[c.code_off..c.code_off + c.code_len]
    }

    #[test]
    fn pattern_matching() {
        assert!(matches_pattern("net/minecraft/*", "net/minecraft/server/Level"));
        assert!(!matches_pattern("net/minecraft/*", "org/bukkit/Server"));
        assert!(matches_pattern("*", "anything"));
        assert!(matches_pattern("a/b/C", "a/b/C"));
    }

    #[test]
    fn no_rules_passthrough() {
        let e = TransformEngine::new();
        assert!(e.apply("x/y/Z", b"bytes").unwrap().is_none());
    }

    #[test]
    fn parse_synthetic_class() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let bytes = c.fin(cl_t, cl_o, vec![method_b(run, v, Some(code_b(code_n, vec![0x03, 0xAC])))]);
        let cf = parse_class(&bytes).unwrap();
        assert_eq!(cf.this_class, cl_t);
        assert_eq!(class_name(&cf, cf.this_class).unwrap(), "Test");
        assert_eq!(cf.methods.len(), 1);
        assert_eq!(cf.methods[0].name, "run");
        assert_eq!(cf.methods[0].descriptor, "()V");
        let code = code_of(&cf, 0);
        assert_eq!(code.code_len, 2);
        assert_eq!(code_bytes(&cf, 0), &[0x03, 0xAC]);
    }

    #[test]
    fn long_entries_take_two_slots() {
        let mut c = Cb::new();
        let t = c.utf8("Test");
        let o = c.utf8("java/lang/Object");
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let cl_t = c.class(t);
        let cl_o = c.class(o);
        let nat = c.nat(run, v);
        let _long = c.long(42);
        let _long2 = c.long(43);
        let mr = c.mref(cl_o, nat); // sits after two 2-slot entries
        let code_n = c.utf8("Code");
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 1,
                    code: vec![OP_INVOKESTATIC, (mr >> 8) as u8, mr as u8, 0xB1],
                    exc: vec![],
                    sub: vec![],
                }),
            )],
        );
        let cf = parse_class(&bytes).unwrap();
        let resolved = resolve_methodref(&cf, mr).unwrap();
        assert_eq!(resolved, ("java/lang/Object".to_string(), "run".to_string(), "()V".to_string()));
    }

    #[test]
    fn method_entry_inserts_invokestatic() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let lnt = c.utf8("LineNumberTable");
        // two line entries: start_pc 0 and 1; the second must shift by +3
        let lnt_payload = vec![0, 2, 0, 0, 0, 1, 0, 1, 0, 2];
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 0,
                    code: vec![0x03, 0xAC],
                    exc: vec![],
                    sub: vec![(lnt, lnt_payload)],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(hook());
        let out = e.apply("Test", &bytes).unwrap().unwrap().bytes;
        if out.len() != bytes.len() + 50 {
            eprintln!("orig {} out {}", bytes.len(), out.len());
            eprintln!("orig: {:02x?}", bytes);
            eprintln!("out:  {:02x?}", out);
        }
        let cf = parse_class(&out).unwrap();
        assert_eq!(class_name(&cf, cf.this_class).unwrap(), "Test");
        let code = code_of(&cf, 0);
        assert_eq!(code.code_len, 5);
        assert_eq!(code_bytes(&cf, 0).len(), 5);
        assert_eq!(code_bytes(&cf, 0)[0], OP_INVOKESTATIC);
        assert_eq!(&code_bytes(&cf, 0)[3..], &[0x03, 0xAC]);
        let mr = u16::from_be_bytes([code_bytes(&cf, 0)[1], code_bytes(&cf, 0)[2]]);
        assert_eq!(
            resolve_methodref(&cf, mr).unwrap(),
            ("dev/crussty/hooks/TickHook".to_string(), "onEntry".to_string(), "()V".to_string())
        );
        // LineNumberTable: (0,1) unchanged, (1,2) -> (4,2)
        let lnt_sub = &code.sub.iter().find(|s| s.name == "LineNumberTable").unwrap();
        assert_eq!(&out[lnt_sub.off..lnt_sub.off + lnt_sub.len], &[0, 2, 0, 0, 0, 1, 0, 4, 0, 2]);
    }

    #[test]
    fn method_entry_skips_methods_without_code() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        // no Code attribute -> abstract/native-like -> skipped
        let bytes = c.fin(cl_t, cl_o, vec![method_b(run, v, None)]);
        let e = TransformEngine::new();
        e.register(hook());
        assert!(e.apply("Test", &bytes).unwrap().is_none());
    }

    #[test]
    fn method_entry_is_idempotent() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let bytes = c.fin(cl_t, cl_o, vec![method_b(run, v, Some(code_b(code_n, vec![0xB1])))]);
        let e = TransformEngine::new();
        e.register(hook());
        let once = e.apply("Test", &bytes).unwrap().unwrap().bytes;
        // re-applying on the transformed bytes must produce no further change
        assert!(e.apply("Test", &once).unwrap().is_none());
    }

    #[test]
    fn before_call_inserts_and_fixes_branches() {
        let mut c = Cb::new();
        let t = c.utf8("Test");
        let o = c.utf8("java/lang/Object");
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let ps = c.utf8("java/io/PrintStream");
        let pln = c.utf8("println");
        let pln_desc = c.utf8("(Ljava/lang/String;)V");
        let cl_t = c.class(t);
        let cl_o = c.class(o);
        let cl_ps = c.class(ps);
        let nat = c.nat(pln, pln_desc);
        let call = c.mref(cl_ps, nat);
        let code_n = c.utf8("Code");
        // offsets: 0 call(3), 3 goto -> 6, 6 iconst_0, 7 ireturn
        let code = vec![OP_INVOKESTATIC, (call >> 8) as u8, call as u8, 0xA7, 0x00, 0x03, 0x03, 0xAC];
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 1,
                    code,
                    exc: vec![],
                    sub: vec![],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(Rule::new(
            "Test",
            "run",
            "()V",
            Injection::BeforeCall("java/io/PrintStream.println:(Ljava/lang/String;)V".to_string()),
            "dev.crussty.hooks.TickHook.onEntry",
        ));
        let out = e.apply("Test", &bytes).unwrap().unwrap().bytes;
        let cf = parse_class(&out).unwrap();
        let code = code_of(&cf, 0);
        assert_eq!(code.code_len, 11);
        let cb = code_bytes(&cf, 0);
        // hook, original call, goto (operand 3 unchanged: both shifted), tail
        assert_eq!(cb[0], OP_INVOKESTATIC);
        assert_eq!(&cb[3..6], &[OP_INVOKESTATIC, (call >> 8) as u8, call as u8]);
        assert_eq!(&cb[6..9], &[0xA7, 0x00, 0x03]);
        assert_eq!(&cb[9..], &[0x03, 0xAC]);
        let hook_mr = u16::from_be_bytes([cb[1], cb[2]]);
        assert_eq!(
            resolve_methodref(&cf, hook_mr).unwrap(),
            ("dev/crussty/hooks/TickHook".to_string(), "onEntry".to_string(), "()V".to_string())
        );
    }

    #[test]
    fn before_call_owner_agnostic_and_no_match() {
        let mut c = Cb::new();
        let t = c.utf8("Test");
        let o = c.utf8("java/lang/Object");
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let ps = c.utf8("java/io/PrintStream");
        let pln = c.utf8("println");
        let pln_desc = c.utf8("(Ljava/lang/String;)V");
        let cl_t = c.class(t);
        let cl_o = c.class(o);
        let cl_ps = c.class(ps);
        let nat = c.nat(pln, pln_desc);
        let call = c.mref(cl_ps, nat);
        let code_n = c.utf8("Code");
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 1,
                    code: vec![OP_INVOKESTATIC, (call >> 8) as u8, call as u8, 0xB1],
                    exc: vec![],
                    sub: vec![],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(Rule::new(
            "Test",
            "*",
            "*",
            Injection::BeforeCall("println:(Ljava/lang/String;)V".to_string()),
            "dev.crussty.hooks.TickHook.onEntry",
        ));
        assert!(e.apply("Test", &bytes).unwrap().is_some());
        let e2 = TransformEngine::new();
        e2.register(Rule::new(
            "Test",
            "*",
            "*",
            Injection::BeforeCall("foo/Bar.baz:()V".to_string()),
            "dev.crussty.hooks.TickHook.onEntry",
        ));
        assert!(e2.apply("Test", &bytes).unwrap().is_none());
    }

    #[test]
    fn stackmap_frame_delta_fixed() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let smt = c.utf8("StackMapTable");
        // goto 4 (frame at 4); frame delta 4 must become 7
        let code = vec![0xA7, 0x00, 0x04, 0x03, 0xB1];
        let smt_payload = vec![0, 1, 4];
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 0,
                    code,
                    exc: vec![],
                    sub: vec![(smt, smt_payload)],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(hook());
        let out = e.apply("Test", &bytes).unwrap().unwrap().bytes;
        let cf = parse_class(&out).unwrap();
        let code = code_of(&cf, 0);
        assert_eq!(code.code_len, 8);
        // goto still targets the iconst_0 (now at 7): operand stays 4
        assert_eq!(&code_bytes(&cf, 0)[3..6], &[0xA7, 0x00, 0x04]);
        let smt_sub = code.sub.iter().find(|s| s.name == "StackMapTable").unwrap();
        assert_eq!(&out[smt_sub.off..smt_sub.off + smt_sub.len], &[0, 1, 7]);
    }

    #[test]
    fn stackmap_delta_grows_to_extended() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let smt = c.utf8("StackMapTable");
        // goto 62; iconst_0 at 62; frame at 62 (delta 62). +3 -> 65 -> extended.
        let mut code = vec![0xA7, 0x00, 0x3E];
        code.extend(std::iter::repeat_n(0x00, 59)); // nops at 3..62
        code.extend_from_slice(&[0x03, 0xB1]);
        let smt_payload = vec![0, 1, 62];
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 0,
                    code,
                    exc: vec![],
                    sub: vec![(smt, smt_payload)],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(hook());
        let out = e.apply("Test", &bytes).unwrap().unwrap().bytes;
        let cf = parse_class(&out).unwrap();
        let code = code_of(&cf, 0);
        // 64-byte body + 3-byte hook
        assert_eq!(code.code_len, 67);
        let smt_sub = code.sub.iter().find(|s| s.name == "StackMapTable").unwrap();
        // same_frame_extended (251) + u2 65; attribute payload grew 3 -> 5
        assert_eq!(smt_sub.len, 5);
        assert_eq!(&out[smt_sub.off..smt_sub.off + smt_sub.len], &[0, 1, 251, 0, 65]);
    }

    #[test]
    fn exception_table_fixed() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 0,
                    code: vec![0x03, 0xAC],
                    exc: vec![[1, 2, 1, 0]],
                    sub: vec![],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(hook());
        let out = e.apply("Test", &bytes).unwrap().unwrap().bytes;
        let cf = parse_class(&out).unwrap();
        let code = code_of(&cf, 0);
        // start_pc 1 -> 4, end_pc 2 -> 5 (== new code_length), handler_pc 1 -> 4
        let (s_off, e_off, h_off) = code.exc[0];
        assert_eq!(u16_at(&out, s_off, "t").unwrap(), 4);
        assert_eq!(u16_at(&out, e_off, "t").unwrap(), 5);
        assert_eq!(u16_at(&out, h_off, "t").unwrap(), 4);
    }

    #[test]
    fn parse_errors_never_panic() {
        assert!(parse_class(b"").is_err());
        assert!(parse_class(b"\xca\xfe\xba\xbe").is_err());
        assert!(parse_class(b"junk").is_err());
        // cp_count declares more entries than exist
        let mut short = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52, 0, 20];
        short.extend_from_slice(&[0; 2]);
        assert!(parse_class(&short).is_err());
        // unknown constant pool tag
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        c.cp.push((99, vec![1, 2, 3]));
        let bad = c.fin(cl_t, cl_o, vec![method_b(run, v, Some(code_b(3, vec![0xB1])))]);
        assert!(parse_class(&bad).is_err());
        // truncated utf8 (declared length beyond data)
        let mut c2 = Cb::new();
        c2.cp.push((TAG_UTF8, vec![0, 100]));
        let bad2 = c2.fin(cl_t, cl_o, vec![]);
        assert!(parse_class(&bad2).is_err());
        // truncated instruction
        let (mut c3, cl_t3, cl_o3) = basic_cb();
        let run3 = c3.utf8("run");
        let v3 = c3.utf8("()V");
        let code_n3 = c3.utf8("Code");
        let bad3 = c3.fin(
            cl_t3,
            cl_o3,
            vec![method_b(
                run3,
                v3,
                Some(CodeB {
                    code_attr_name: code_n3,
                    max_stack: 1,
                    max_locals: 0,
                    code: vec![0xA7, 0x00], // goto with missing operand
                    exc: vec![],
                    sub: vec![],
                }),
            )],
        );
        assert!(parse_class(&bad3).unwrap().methods[0].code.is_some());
        let e = TransformEngine::new();
        e.register(hook());
        assert!(e.apply("Test", &bad3).is_err());
    }

    #[test]
    fn rule_method_filter_applies() {
        let (mut c, cl_t, cl_o) = basic_cb();
        let run = c.utf8("run");
        let v = c.utf8("()V");
        let code_n = c.utf8("Code");
        let bytes = c.fin(
            cl_t,
            cl_o,
            vec![method_b(
                run,
                v,
                Some(CodeB {
                    code_attr_name: code_n,
                    max_stack: 1,
                    max_locals: 0,
                    code: vec![0xB1],
                    exc: vec![],
                    sub: vec![],
                }),
            )],
        );
        let e = TransformEngine::new();
        e.register(Rule::new("Test", "other", "()V", Injection::MethodEntry, "x.y.z.h"));
        assert!(e.apply("Test", &bytes).unwrap().is_none());
        let e2 = TransformEngine::new();
        e2.register(Rule::new("Test", "run", "(I)V", Injection::MethodEntry, "x.y.z.h"));
        assert!(e2.apply("Test", &bytes).unwrap().is_none());
    }

    // --- real-world class from the kernel jar ------------------------------

    #[test]
    fn real_jar_class_roundtrip() {
        let jar_path = "/home/btw/crussty-dist/v2/versions/purpur-1.21.10.jar";
        if !std::path::Path::new(jar_path).exists() {
            eprintln!("skipping: {} missing", jar_path);
            return;
        }
        use std::io::Read as _;
        let file = std::fs::File::open(jar_path).unwrap();
        let mut arc = zip::ZipArchive::new(file).unwrap();
        let mut candidates: Vec<(u64, String)> = (0..arc.len())
            .filter_map(|i| {
                let f = arc.by_index(i).ok()?;
                f.name()
                    .ends_with(".class")
                    .then(|| (f.size(), f.name().to_string()))
            })
            .collect();
        candidates.sort_by_key(|(size, _)| *size);
        let mut tested = 0usize;
        for (_, name) in candidates {
            let mut f = arc.by_name(&name).unwrap();
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes).unwrap();
            let cf = match parse_class(&bytes) {
                Ok(cf) => cf,
                Err(_) => continue,
            };
            if !cf.methods.iter().any(|m| m.code.is_some()) {
                continue;
            }
            tested += 1;
            let internal_name = name.strip_suffix(".class").unwrap();
            let e = TransformEngine::new();
            e.register(Rule::new("*", "*", "*", Injection::MethodEntry, "dev.crussty.hooks.TickHook.onEntry"));
            let out = e.apply(internal_name, &bytes).expect("apply to real class").expect("changed");
            let cf2 = parse_class(&out.bytes).expect("re-parse transformed class");
            assert_eq!(class_name(&cf2, cf2.this_class).unwrap(), class_name(&cf, cf.this_class).unwrap());
            assert_eq!(cf2.methods.len(), cf.methods.len());
            for (a, b) in cf.methods.iter().zip(&cf2.methods) {
                assert_eq!(a.name, b.name);
                assert_eq!(a.descriptor, b.descriptor);
                match (&a.code, &b.code) {
                    (Some(ca), Some(cb)) => {
                        assert_eq!(cb.code_len, ca.code_len + 3);
                        assert_eq!(out.bytes[cb.code_off], OP_INVOKESTATIC);
                        assert_eq!(
                            &out.bytes[cb.code_off..cb.code_off + cb.code_len][3..],
                            &bytes[ca.code_off..ca.code_off + ca.code_len]
                        );
                    }
                    (None, None) => {}
                    _ => panic!("code presence changed"),
                }
            }
            if tested >= 2 {
                return;
            }
        }
        panic!("no suitable class found in jar");
    }
}
