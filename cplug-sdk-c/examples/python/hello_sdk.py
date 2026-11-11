"""Crussty module body written in Python, driving the SDK C binding.

The runtime dlopens libhello_py_sdk.so (C shim) which calls `cplugin_init` in
this file. Everything below is plain Python over ctypes: the hooks, the
kernel-ready notification, main-thread dispatch and logging all go through
libcplug_sdk_c.so — no JNI, no Python C-API.

Deploy: libhello_py_sdk.so + libcplug_sdk_c.so + hello_sdk.py + cplugin.json
"""

import ctypes
import os

# The SDK is dlopen'd by the shim (RTLD_LOCAL), but we keep our own handle
# to the same file so ctypes gets function pointers straight from it.
SDK = ctypes.CDLL(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                               "libcplug_sdk_c.so"))

def sdk(name, restype, argtypes):
    f = getattr(SDK, name)
    f.restype = restype
    f.argtypes = argtypes
    return f

sdk_init = sdk("cplug_sdk_init", None, [ctypes.c_void_p, ctypes.c_void_p])
sdk_log_info = sdk("cplug_sdk_log_info", None, [ctypes.c_char_p])
sdk_hook = sdk("cplug_sdk_hook_register", ctypes.c_int32,
               [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_void_p])
sdk_main = sdk("cplug_sdk_run_on_main_thread", ctypes.c_int32,
               [ctypes.c_void_p, ctypes.c_void_p])
sdk_ready = sdk("cplug_sdk_on_kernel_ready", ctypes.c_int32,
                [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_void_p])

HOOK_FN = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_char_p)
READY_FN = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
MAIN_FN = ctypes.CFUNCTYPE(None, ctypes.c_void_p, ctypes.c_void_p)

# Keep the callbacks alive for the whole module lifetime (the SDK never
# unregisters; CPython refcounts would otherwise free them).
_kept = []

@HOOK_FN
def on_class(ctx, name):
    sdk_log_info(b"[hello_sdk] python saw class " + (name or b"?"))
_kept.append(on_class)

@READY_FN
def on_ready(ctx):
    sdk_log_info(b"[hello_sdk] kernel ready (python)")
    sdk_main(None, on_main)
_kept.append(on_ready)

@MAIN_FN
def on_main(ctx, env):
    sdk_log_info(b"[hello_sdk] on the server thread (python)")
_kept.append(on_main)

def cplugin_init(api_addr, vm_addr, options):
    # (api, vm are raw addresses passed from the C shim)
    sdk_init(ctypes.c_void_p(api_addr), ctypes.c_void_p(vm_addr))
    sdk_hook(b"org/bukkit/**", None, on_class)
    sdk_ready(b"org/bukkit/Bukkit", None, on_ready)
    return 0