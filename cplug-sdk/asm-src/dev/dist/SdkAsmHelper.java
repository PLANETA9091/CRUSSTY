package dev.dist;

import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.io.IOException;

import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassVisitor;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.Label;
import org.objectweb.asm.MethodVisitor;
import org.objectweb.asm.Opcodes;

/**
 * ASM-based classfile rewrite helper (milestone 3).
 *
 * Pure-Rust patching (cplug_sdk::weave) cannot recompute StackMapTable
 * frames for major-65 classes; this helper runs inside the JVM where ASM
 * (on the kernel's classpath — libraries/org/ow2/asm) can re-frame any
 * transformation with COMPUTE_FRAMES.
 *
 * The class is defined by the SDK into the loader of the TARGET class
 * (parent chain reaches the system loader where ASM lives), so `this_class`
 * must be unique per module — the SDK renames it before defining.
 *
 * API: `rewrite(byte[] cls, byte[] spec)` returns the rewritten class, or
 * null on failure (`lastError()` carries the reason). Spec (big-endian):
 *   u8  version = 1
 *   u8  op
 *   op 1 REPLACE_BODY:
 *     String methodName (u16 len + UTF-8 bytes)
 *     String methodDesc
 *     String bridgeOwner (internal name)
 *     String bridgeName
 *     String bridgeDesc
 *     u8  argCount
 *     per arg:
 *       u16 slot, u8 typeChar    — local (0 = this for instance methods),
 *                                  typeChar selects the load opcode
 *                                  (I/J/D/F/Z/L/[)
 *       or: u16 slot(ignored), u8 '@', String fieldName, String fieldDesc
 *                                  — emit `aload 0; getfield this.fieldName:
 *                                  fieldDesc` (reads a private field of the
 *                                  patched class from inside its own method;
 *                                  changing the field's ACCESS FLAGS in the
 *                                  retransformed class file is NOT allowed —
 *                                  the JVM rejects it with
 *                                  UNSUPPORTED_REDEFINITION_SCHEMA_CHANGED)
 *   op 2 MAKE_FIELDS_PUBLIC: widen access so a bridge in another class can
 *     read kernel state. NOTE: only usable for non-retransform rewriting
 *     (e.g. warmup); a retransform target must not change field modifiers.
 *     u8   fieldCount
 *     String name x fieldCount   (field name, u16 len + UTF-8)
 *     String desc x fieldCount    (field descriptor)
 *   op 3 SKIP_GUARD:
 *     String methodName, String methodDesc, String checkClass,
 *     String checkName, String checkDesc, u16 target, u8 hasInitLocal
 *     [u8 localSlot, String initName, String initDesc], u8 hasEndPush
 *     [u8 endOpcode]
 *   op 4 EARLY_RETURN_GUARD:
 *     String methodName, String methodDesc, String checkClass,
 *     String checkName, String checkDesc
 */
public final class SdkAsmHelper {

    private static volatile String LAST_ERROR = "";

    private SdkAsmHelper() {
    }

    public static String lastError() {
        return LAST_ERROR;
    }

    public static byte[] rewrite(byte[] cls, byte[] spec) {
        LAST_ERROR = "";
        try {
            DataInputStream in = new DataInputStream(new ByteArrayInputStream(spec));
            int version = in.readUnsignedByte();
            if (version != 1) {
                LAST_ERROR = "unsupported spec version " + version;
                return null;
            }
            int op = in.readUnsignedByte();
            switch (op) {
                case 1:
                    return replaceBody(cls, in);
                case 2:
                    return makeFieldsPublic(cls, in);
                case 3:
                    return insertSkipGuard(cls, in);
                case 4:
                    return insertEarlyReturnGuard(cls, in);
                default:
                    LAST_ERROR = "unknown op " + op;
                    return null;
            }
        } catch (Throwable t) {
            LAST_ERROR = t.toString();
            return null;
        }
    }

    // ------------------------------------------------------------------
    // op 1: replace a method body with invokestatic to a bridge, then
    // return. The bridge descriptor's return type must equal the method's.
    // ------------------------------------------------------------------
    private static byte[] replaceBody(byte[] cls, DataInputStream in) throws IOException {
        String methodName = in.readUTF();
        String methodDesc = in.readUTF();
        String bridgeOwner = in.readUTF();
        String bridgeName = in.readUTF();
        String bridgeDesc = in.readUTF();
        int argCount = in.readUnsignedByte();
        int[] slots = new int[argCount];
        char[] types = new char[argCount];
        String[] fieldNames = new String[argCount];
        String[] fieldDescs = new String[argCount];
        for (int i = 0; i < argCount; i++) {
            slots[i] = in.readUnsignedShort();
            types[i] = (char) in.readUnsignedByte();
            if (types[i] == '@') {
                fieldNames[i] = in.readUTF();
                fieldDescs[i] = in.readUTF();
            }
        }

        ClassReader cr = new ClassReader(cls);
        ClassWriter cw = new ClassWriter(cr, ClassWriter.COMPUTE_FRAMES);
        cr.accept(new ClassVisitor(Opcodes.ASM9, cw) {
            @Override
            public MethodVisitor visitMethod(int access, String name, String desc,
                    String signature, String[] exceptions) {
                MethodVisitor mv = super.visitMethod(access, name, desc, signature, exceptions);
                if (!name.equals(methodName) || !desc.equals(methodDesc)) {
                    return mv;
                }
                // Consuming wrapper: emits the new body in visitCode, swallows
                // every other instruction event so the original body never
                // reaches the writer. visitEnd still forwards to close the
                // method's attribute list.
                return new MethodVisitor(Opcodes.ASM9, mv) {
                    @Override
                    public void visitCode() {
                        mv.visitCode();
                        for (int i = 0; i < argCount; i++) {
                            if (types[i] == '@') {
                                // `this.<field>` read inside the patched
                                // class's own method: private access is
                                // legal here (same class).
                                mv.visitVarInsn(Opcodes.ALOAD, 0);
                                mv.visitFieldInsn(Opcodes.GETFIELD,
                                        cr.getClassName(), fieldNames[i], fieldDescs[i]);
                            } else {
                                mv.visitVarInsn(loadOpcode(types[i]), slots[i]);
                            }
                        }
                        mv.visitMethodInsn(Opcodes.INVOKESTATIC,
                                bridgeOwner, bridgeName, bridgeDesc, false);
                        mv.visitInsn(returnOpcode(methodDesc.charAt(methodDesc.length() - 1)));
                        // COMPUTE_FRAMES also recomputes maxs; values ignored.
                        mv.visitMaxs(0, 0);
                    }

                    @Override
                    public void visitInsn(int i) { }

                    @Override
                    public void visitIntInsn(int i, int v) { }

                    @Override
                    public void visitVarInsn(int i, int v) { }

                    @Override
                    public void visitTypeInsn(int i, String s) { }

                    @Override
                    public void visitFieldInsn(int i, String a, String b, String c) { }

                    @Override
                    public void visitMethodInsn(int i, String a, String b, String c, boolean d) { }

                    @Override
                    public void visitInvokeDynamicInsn(String a, String b, org.objectweb.asm.Handle h,
                            Object... o) { }

                    @Override
                    public void visitJumpInsn(int i, Label l) { }

                    @Override
                    public void visitLabel(Label l) { }

                    @Override
                    public void visitLdcInsn(Object o) { }

                    @Override
                    public void visitIincInsn(int i, int v) { }

                    @Override
                    public void visitTableSwitchInsn(int a, int b, Label l, Label... ls) { }

                    @Override
                    public void visitLookupSwitchInsn(Label l, int[] k, Label[] ls) { }

                    @Override
                    public void visitMultiANewArrayInsn(String s, int d) { }

                    @Override
                    public void visitTryCatchBlock(Label a, Label b, Label c, String s) { }

                    @Override
                    public void visitLineNumber(int i, Label l) { }

                    @Override
                    public void visitLocalVariable(String a, String b, String c,
                            Label l, Label m, int i) { }

                    @Override
                    public void visitFrame(int a, int b, Object[] c, int d, Object[] e) { }

                    @Override
                    public void visitMaxs(int a, int b) { }
                };
            }
        }, 0);
        return cw.toByteArray();
    }

    // ------------------------------------------------------------------
    // op 2: widen the access of a list of fields to `public` so bridge
    // classes defined in another class can read kernel state directly.
    // ------------------------------------------------------------------
    private static byte[] makeFieldsPublic(byte[] cls, DataInputStream in) throws IOException {
        int count = in.readUnsignedByte();
        String[] names = new String[count];
        String[] descs = new String[count];
        for (int i = 0; i < count; i++) {
            names[i] = in.readUTF();
            descs[i] = in.readUTF();
        }

        ClassReader cr = new ClassReader(cls);
        ClassWriter cw = new ClassWriter(cr, ClassWriter.COMPUTE_FRAMES);
        cr.accept(new ClassVisitor(Opcodes.ASM9, cw) {
            @Override
            public org.objectweb.asm.FieldVisitor visitField(int access, String name,
                    String desc, String signature, Object value) {
                int newAccess = access;
                for (int i = 0; i < count; i++) {
                    if (names[i].equals(name) && descs[i].equals(desc)) {
                        newAccess &= ~(Opcodes.ACC_PRIVATE | Opcodes.ACC_PROTECTED);
                        newAccess |= Opcodes.ACC_PUBLIC;
                        break;
                    }
                }
                return super.visitField(newAccess, name, desc, signature, value);
            }
        }, 0);
        return cw.toByteArray();
    }

    // ------------------------------------------------------------------
    // op 3: conditional mid-body skip guard (c-entities L2a, resting items).
    //
    // Emits at the method start:
    //   aload_0
    //   invokestatic checkClass.checkName(checkDesc)   // (this-type)Z
    //   ifeq L0                                        // false -> original body
    //   <skip setup>                                   // caller-specified:
    //     [initLocal: aload_0; invokevirtual initName()T; astore slot]
    //     [endPush: endOpcode]                         // e.g. iconst_0
    //   goto LT                                        // LT = target + delta
    //   L0: <original body>
    //
    // `target` is a bytecode offset in the ORIGINAL method the skip path
    // jumps to (e.g. ItemEntity.tick's merge gate at 432, past the physics).
    // COMPUTE_FRAMES re-derives every frame including LT (ASM abstractly
    // interprets both paths), so the caller does not hand-build frames.
    // The caller must guarantee the skip path's LOCAL TYPES match the
    // original flow at LT (that is what initLocal is for) and that the
    // operand stack at LT matches (endPush).
    // ------------------------------------------------------------------
    private static byte[] insertSkipGuard(byte[] cls, DataInputStream in) throws IOException {
        String methodName = in.readUTF();
        String methodDesc = in.readUTF();
        String checkClass = in.readUTF();
        String checkName = in.readUTF();
        String checkDesc = in.readUTF();
        int target = in.readUnsignedShort();
        boolean initLocal = in.readBoolean();
        int localSlot = in.readUnsignedByte();
        String initName = initLocal ? in.readUTF() : "";
        String initDesc = initLocal ? in.readUTF() : "";
        boolean endPush = in.readBoolean();
        int endOpcode = endPush ? in.readUnsignedByte() : 0;

        org.objectweb.asm.tree.ClassNode cn = new org.objectweb.asm.tree.ClassNode();
        new ClassReader(cls).accept(cn, 0);
        boolean done = false;
        for (Object mo : cn.methods) {
            org.objectweb.asm.tree.MethodNode mn = (org.objectweb.asm.tree.MethodNode) mo;
            if (!mn.name.equals(methodName) || !mn.desc.equals(methodDesc)) {
                continue;
            }
            // Target selection. Runtime class bytes differ from the shipped
            // jar (Paper re-transforms classes at load), so a hard offset is
            // fragile. target == 0 selects the built-in merge-interval
            // pattern of ItemEntity.tick: the istore_2 of
            //   iconst_0, istore_2, iload_2, ifeq, iconst_2, goto,
            //   bipush 40, istore_3
            // (the 2-or-40 merge interval; the skip path lands on the
            // istore_2 with iconst_0 already pushed -> interval 40 = idle
            // merge cadence, exactly what a resting item computes).
            org.objectweb.asm.tree.AbstractInsnNode targetNode = null;
            if (target == 0) {
                for (org.objectweb.asm.tree.AbstractInsnNode n = mn.instructions.getFirst();
                        n != null; n = n.getNext()) {
                    if (!isIntStore(n, 2)) {
                        continue;
                    }
                    // neighbors skip Label/Frame nodes (they carry no bytecode
                    // but sit between real instructions in the tree)
                    org.objectweb.asm.tree.AbstractInsnNode p1 = realPrev(n);
                    org.objectweb.asm.tree.AbstractInsnNode p2 = realNext(n);
                    org.objectweb.asm.tree.AbstractInsnNode p3 = realNext(p2);
                    org.objectweb.asm.tree.AbstractInsnNode p4 = realNext(p3);
                    org.objectweb.asm.tree.AbstractInsnNode p5 = realNext(p4);
                    org.objectweb.asm.tree.AbstractInsnNode p6 = realNext(p5);
                    org.objectweb.asm.tree.AbstractInsnNode p7 = realNext(p6);
                    org.objectweb.asm.tree.AbstractInsnNode p8 = realNext(p7);
                    if (p1 == null || p2 == null || p8 == null) {
                        continue;
                    }
                    // the interval store/load may be the compact form
                    // (istore_2/iload_2) OR the indexed form (istore 2 /
                    // iload 2) — Paper's runtime transforms use the indexed
                    // opcodes while javap prints them identically
                    boolean ok = p1.getOpcode() == Opcodes.ICONST_0
                            && isIntLoad(p2, 2)
                            && p3.getOpcode() == Opcodes.IFEQ
                            && p4.getOpcode() == Opcodes.ICONST_2
                            && p5.getOpcode() == Opcodes.GOTO
                            && p6.getOpcode() == Opcodes.BIPUSH
                            && ((org.objectweb.asm.tree.IntInsnNode) p6).operand == 40
                            && isIntStore(p7, 3)
                            && (p8.getOpcode() == Opcodes.ALOAD || p8.getOpcode() == 0x2a /* aload_0 */);
                    if (ok) {
                        targetNode = n;
                        break;
                    }
                }
                if (targetNode == null) {
                    LAST_ERROR = "skip-guard: merge-interval pattern not found";
                    return null;
                }
            } else {
                // fallback: hard offset in the original bytecode (size table
                // walk; kept for non-item weaves where the bytes are stable)
                int pos = 0;
                for (org.objectweb.asm.tree.AbstractInsnNode n = mn.instructions.getFirst();
                        n != null; n = n.getNext()) {
                    if (pos == target) {
                        targetNode = n;
                        break;
                    }
                    pos += insnSize(n);
                    if (pos > target) {
                        LAST_ERROR = "skip-guard target " + target
                                + " not on an instruction boundary (landed in " + pos + ")";
                        return null;
                    }
                }
                if (targetNode == null) {
                    LAST_ERROR = "skip-guard target " + target + " past method end";
                    return null;
                }
            }
            org.objectweb.asm.tree.LabelNode lt = new org.objectweb.asm.tree.LabelNode();
            mn.instructions.insertBefore(targetNode, lt);
            org.objectweb.asm.tree.InsnList guard = new org.objectweb.asm.tree.InsnList();
            guard.add(new org.objectweb.asm.tree.VarInsnNode(Opcodes.ALOAD, 0));
            guard.add(new org.objectweb.asm.tree.MethodInsnNode(Opcodes.INVOKESTATIC,
                    checkClass, checkName, checkDesc, false));
            org.objectweb.asm.tree.LabelNode l0 = new org.objectweb.asm.tree.LabelNode();
            guard.add(new org.objectweb.asm.tree.JumpInsnNode(Opcodes.IFEQ, l0));
            if (initLocal) {
                guard.add(new org.objectweb.asm.tree.VarInsnNode(Opcodes.ALOAD, 0));
                guard.add(new org.objectweb.asm.tree.MethodInsnNode(Opcodes.INVOKEVIRTUAL,
                        cn.name, initName, initDesc, false));
                guard.add(new org.objectweb.asm.tree.VarInsnNode(Opcodes.ASTORE, localSlot));
            }
            if (endPush) {
                guard.add(new org.objectweb.asm.tree.InsnNode(endOpcode));
            }
            guard.add(new org.objectweb.asm.tree.JumpInsnNode(Opcodes.GOTO, lt));
            guard.add(l0);
            mn.instructions.insert(guard);
            done = true;
            break;
        }
        if (!done) {
            LAST_ERROR = "skip-guard: method " + methodName + methodDesc + " not found";
            return null;
        }
        ClassWriter cw = new ClassWriter(ClassWriter.COMPUTE_FRAMES);
        cn.accept(cw);
        return cw.toByteArray();
    }

    // ------------------------------------------------------------------
    // op 4: conditional early-return guard (c-entities L3, Rust merge
    // broad-phase; L7 dispatch takeover).
    //
    // Emits at the method start:
    //   <receiver local>
    //   invokestatic checkClass.checkName(checkDesc)   // (receiver-type)Z
    //   ifeq L0                                        // false -> original body
    //   <return opcode of the method's return type>    // true -> done
    //   L0: <original body>
    //
    // RECEIVER LOCAL SELECTION (L7): the guard predicate receives `this`
    // (local 0) for methods with no reference parameters (e.g. the
    // ItemEntity.tick()V guard). When the method's FIRST parameter is a
    // reference type (L...; or [...), the predicate receives THAT argument
    // (local 1) instead — the dispatch guard on
    // ServerLevel.tickNonPassenger(Entity)V must see the ENTITY, not the
    // level. Primitive-first-arg methods keep the old local-0 behavior.
    //
    // Used for ItemEntity.mergeWithNeighbours()V: the fast path performs the
    // whole neighbour merge through the Rust index and returns true; false
    // falls through to the vanilla body (exact mirror).
    // COMPUTE_FRAMES re-derives all frames; the guard consumes no locals and
    // leaves the operand stack empty on both paths.
    // ------------------------------------------------------------------
    private static byte[] insertEarlyReturnGuard(byte[] cls, DataInputStream in) throws IOException {
        String methodName = in.readUTF();
        String methodDesc = in.readUTF();
        String checkClass = in.readUTF();
        String checkName = in.readUTF();
        String checkDesc = in.readUTF();

        org.objectweb.asm.tree.ClassNode cn = new org.objectweb.asm.tree.ClassNode();
        new ClassReader(cls).accept(cn, 0);
        boolean done = false;
        for (Object mo : cn.methods) {
            org.objectweb.asm.tree.MethodNode mn = (org.objectweb.asm.tree.MethodNode) mo;
            if (!mn.name.equals(methodName) || !mn.desc.equals(methodDesc)) {
                continue;
            }
            org.objectweb.asm.tree.InsnList guard = new org.objectweb.asm.tree.InsnList();
            String dsc = mn.desc;
            int recvLocal = 0;
            if (dsc.length() > 1 && (dsc.charAt(1) == 'L' || dsc.charAt(1) == '[')) {
                recvLocal = 1;
            }
            guard.add(new org.objectweb.asm.tree.VarInsnNode(Opcodes.ALOAD, recvLocal));
            guard.add(new org.objectweb.asm.tree.MethodInsnNode(Opcodes.INVOKESTATIC,
                    checkClass, checkName, checkDesc, false));
            org.objectweb.asm.tree.LabelNode l0 = new org.objectweb.asm.tree.LabelNode();
            guard.add(new org.objectweb.asm.tree.JumpInsnNode(Opcodes.IFEQ, l0));
            guard.add(new org.objectweb.asm.tree.InsnNode(
                    returnOpcode(methodDesc.charAt(methodDesc.length() - 1))));
            guard.add(l0);
            mn.instructions.insert(guard);
            done = true;
            break;
        }
        if (!done) {
            LAST_ERROR = "early-return-guard: method " + methodName + methodDesc + " not found";
            return null;
        }
        ClassWriter cw = new ClassWriter(ClassWriter.COMPUTE_FRAMES);
        cn.accept(cw);
        return cw.toByteArray();
    }

    private static boolean isIntStore(org.objectweb.asm.tree.AbstractInsnNode n, int var) {
        if (n instanceof org.objectweb.asm.tree.VarInsnNode) {
            org.objectweb.asm.tree.VarInsnNode v = (org.objectweb.asm.tree.VarInsnNode) n;
            return (v.getOpcode() == 0x3d + var && var <= 3) || (v.getOpcode() == Opcodes.ISTORE && v.var == var);
        }
        return false;
    }

    private static boolean isIntLoad(org.objectweb.asm.tree.AbstractInsnNode n, int var) {
        if (n instanceof org.objectweb.asm.tree.VarInsnNode) {
            org.objectweb.asm.tree.VarInsnNode v = (org.objectweb.asm.tree.VarInsnNode) n;
            return (v.getOpcode() == 0x1a + var && var <= 3) || (v.getOpcode() == Opcodes.ILOAD && v.var == var);
        }
        return false;
    }

    private static org.objectweb.asm.tree.AbstractInsnNode realPrev(
            org.objectweb.asm.tree.AbstractInsnNode n) {
        org.objectweb.asm.tree.AbstractInsnNode p = n == null ? null : n.getPrevious();
        while (p != null && p.getOpcode() == -1) {
            p = p.getPrevious();
        }
        return p;
    }

    private static org.objectweb.asm.tree.AbstractInsnNode realNext(
            org.objectweb.asm.tree.AbstractInsnNode n) {
        org.objectweb.asm.tree.AbstractInsnNode q = n == null ? null : n.getNext();
        while (q != null && q.getOpcode() == -1) {
            q = q.getNext();
        }
        return q;
    }

    /** Bytecode length of a tree instruction (JVMS 6.5; no tableswitch/JSR
     *  handling — the guard target is never inside one for our weaves, and a
     *  wrong total is caught by the boundary check above). */
    private static int insnSize(org.objectweb.asm.tree.AbstractInsnNode n) {
        int op = n.getOpcode();
        if (op == -1) {
            return 0; // labels / frames / line numbers carry no bytecode
        }
        switch (op) {
            case Opcodes.BIPUSH:
            case Opcodes.SIPUSH:
            case Opcodes.LDC:
            case Opcodes.ILOAD: case Opcodes.LLOAD: case Opcodes.FLOAD: case Opcodes.DLOAD:
            case Opcodes.ALOAD:
            case Opcodes.ISTORE: case Opcodes.LSTORE: case Opcodes.FSTORE: case Opcodes.DSTORE:
            case Opcodes.ASTORE:
            case Opcodes.RET:
                return 2; // opcode + index (wide forms are refused by boundary check)
            case Opcodes.IFEQ: case Opcodes.IFNE: case Opcodes.IFLT: case Opcodes.IFGE:
            case Opcodes.IFGT: case Opcodes.IFLE:
            case Opcodes.IF_ICMPEQ: case Opcodes.IF_ICMPNE: case Opcodes.IF_ICMPLT:
            case Opcodes.IF_ICMPGE: case Opcodes.IF_ICMPGT: case Opcodes.IF_ICMPLE:
            case Opcodes.IF_ACMPEQ: case Opcodes.IF_ACMPNE:
            case Opcodes.GOTO:
            case Opcodes.GETSTATIC: case Opcodes.PUTSTATIC:
            case Opcodes.GETFIELD: case Opcodes.PUTFIELD:
            case Opcodes.INVOKEVIRTUAL: case Opcodes.INVOKESPECIAL: case Opcodes.INVOKESTATIC:
            case Opcodes.NEW: case Opcodes.ANEWARRAY: case Opcodes.CHECKCAST: case Opcodes.INSTANCEOF:
            case Opcodes.IINC:
                return 3;
            case 0x14: /* ldc2_w */
            case Opcodes.INVOKEINTERFACE:
            case Opcodes.INVOKEDYNAMIC:
            case 0xc8: /* goto_w */
                return 5;
            case Opcodes.MULTIANEWARRAY:
                return 4;
            case Opcodes.NEWARRAY:
                return 2; // opcode + atype
            default:
                return 1; // consts, stack ops, arithmetic, returns, xload_n/xstore_n...
        }
    }

    private static int loadOpcode(char t) {
        switch (t) {
            case 'J': return Opcodes.LLOAD;
            case 'D': return Opcodes.DLOAD;
            case 'F': return Opcodes.FLOAD;
            case 'I':
            case 'Z':
            case 'B':
            case 'C':
            case 'S': return Opcodes.ILOAD;
            default: return Opcodes.ALOAD; // L... or [
        }
    }

    private static int returnOpcode(char t) {
        switch (t) {
            case 'V': return Opcodes.RETURN;
            case 'J': return Opcodes.LRETURN;
            case 'D': return Opcodes.DRETURN;
            case 'F': return Opcodes.FRETURN;
            case 'I':
            case 'Z':
            case 'B':
            case 'C':
            case 'S': return Opcodes.IRETURN;
            default: return Opcodes.ARETURN; // L... or [
        }
    }
}
