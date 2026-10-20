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

        ClassReader cr = new ClassReader(cls);
        ClassWriter cw = new ClassWriter(cr, ClassWriter.COMPUTE_FRAMES);
        final String owner = cr.getClassName();
        cr.accept(new ClassVisitor(Opcodes.ASM9, cw) {
            @Override
            public MethodVisitor visitMethod(int access, String name, String desc,
                    String signature, String[] exceptions) {
                MethodVisitor mv = super.visitMethod(access, name, desc, signature, exceptions);
                if (!name.equals(methodName) || !desc.equals(methodDesc)) {
                    return mv;
                }
                return new MethodVisitor(Opcodes.ASM9, mv) {
                    private boolean started = false;
                    private Label l0 = new Label();
                    private Label lt = new Label();

                    @Override
                    public void visitCode() {
                        mv.visitCode();
                        // guard: if (!check(this)) goto L0 (run the original)
                        mv.visitVarInsn(Opcodes.ALOAD, 0);
                        mv.visitMethodInsn(Opcodes.INVOKESTATIC,
                                checkClass, checkName, checkDesc, false);
                        mv.visitJumpInsn(Opcodes.IFEQ, l0);
                        // skip-path setup: initialize the local the original
                        // flow expects at the target (types must match), push
                        // the stack value the target consumes, jump.
                        if (initLocal) {
                            mv.visitVarInsn(Opcodes.ALOAD, 0);
                            mv.visitMethodInsn(Opcodes.INVOKEVIRTUAL,
                                    owner, initName, initDesc, false);
                            mv.visitVarInsn(Opcodes.ASTORE, localSlot);
                        }
                        if (endPush) {
                            mv.visitInsn(endOpcode);
                        }
                        mv.visitJumpInsn(Opcodes.GOTO, lt);
                        // original body entry
                        mv.visitLabel(l0);
                        started = true;
                    }

                    // The original body passes through unchanged; only the
                    // FIRST frame (the implicit offset-0 one) must not be
                    // re-emitted: the guard instructions precede it. ASM's
                    // COMPUTE_FRAMES recomputes all frames anyway — we just
                    // swallow the original frame events and let the writer
                    // derive them.
                    @Override
                    public void visitFrame(int a, int b, Object[] c, int d, Object[] e) {
                        mv.visitFrame(a, b, c, d, e);
                    }

                    @Override
                    public void visitLabel(Label label) {
                        mv.visitLabel(label);
                    }

                    // offset tracking to bind LT at the caller's original
                    // bytecode offset: count instruction lengths as they pass.
                    private int pos = 0;

                    @Override
                    public void visitInsn(int opcode) {
                        bindIfTarget(opcode, 1);
                        mv.visitInsn(opcode);
                    }

                    @Override
                    public void visitIntInsn(int opcode, int operand) {
                        bindIfTarget(opcode, opcode == Opcodes.BIPUSH || opcode == Opcodes.SIPUSH ? 2 : 1);
                        mv.visitIntInsn(opcode, operand);
                    }

                    @Override
                    public void visitVarInsn(int opcode, int var) {
                        bindIfTarget(opcode, opcode == Opcodes.RET ? 2 : 1
                                + ((opcode == Opcodes.LLOAD || opcode == Opcodes.DLOAD
                                    || opcode == Opcodes.LSTORE || opcode == Opcodes.DSTORE) ? 1 : 0));
                        mv.visitVarInsn(opcode, var);
                    }

                    @Override
                    public void visitTypeInsn(int opcode, String type) {
                        bindIfTarget(opcode, 3);
                        mv.visitTypeInsn(opcode, type);
                    }

                    @Override
                    public void visitFieldInsn(int opcode, String o, String n, String d) {
                        bindIfTarget(opcode, 3);
                        mv.visitFieldInsn(opcode, o, n, d);
                    }

                    @Override
                    public void visitMethodInsn(int opcode, String o, String n, String d, boolean itf) {
                        bindIfTarget(opcode, opcode == Opcodes.INVOKEINTERFACE ? 5 : 3);
                        mv.visitMethodInsn(opcode, o, n, d, itf);
                    }

                    @Override
                    public void visitInvokeDynamicInsn(String n, String d,
                            org.objectweb.asm.Handle h, Object... bsmArgs) {
                        bindIfTarget(Opcodes.INVOKEDYNAMIC, 5);
                        mv.visitInvokeDynamicInsn(n, d, h, bsmArgs);
                    }

                    @Override
                    public void visitJumpInsn(int opcode, Label label) {
                        bindIfTarget(opcode, opcode == 0xc8 /* goto_w */ ? 5 : 3);
                        mv.visitJumpInsn(opcode, label);
                    }

                    @Override
                    public void visitLdcInsn(Object o) {
                        // offset footprint: LDC = 2 (1 opcode + 1 cpool index),
                        // LDC2_W (Long/Double) = 3
                        int sz = (o instanceof Long || o instanceof Double) ? 3 : 2;
                        bindIfTarget(Opcodes.LDC, sz);
                        mv.visitLdcInsn(o);
                    }

                    @Override
                    public void visitIincInsn(int var, int increment) {
                        bindIfTarget(Opcodes.IINC, 3);
                        mv.visitIincInsn(var, increment);
                    }

                    @Override
                    public void visitTableSwitchInsn(int min, int max, Label dflt, Label... labels) {
                        bindIfTarget(Opcodes.TABLESWITCH, 0); // refuse-by-offset: never hit
                        mv.visitTableSwitchInsn(min, max, dflt, labels);
                    }

                    @Override
                    public void visitLookupSwitchInsn(Label dflt, int[] keys, Label[] labels) {
                        bindIfTarget(Opcodes.LOOKUPSWITCH, 0);
                        mv.visitLookupSwitchInsn(dflt, keys, labels);
                    }

                    @Override
                    public void visitMultiANewArrayInsn(String desc, int dims) {
                        bindIfTarget(Opcodes.MULTIANEWARRAY, 4);
                        mv.visitMultiANewArrayInsn(desc, dims);
                    }

                    private boolean bound = false;

                    private void bindIfTarget(int opcode, int size) {
                        if (!bound && pos == target) {
                            mv.visitLabel(lt);
                            bound = true;
                        }
                        pos += size;
                    }
                };
            }
        }, 0);
        return cw.toByteArray();
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
