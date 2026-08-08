# hello_py — module body written in Python (loaded by the C shim).
#
# The shim imports this module and calls make_hook() for the class-file hook.
# A module author writes everything here in Python; JVM-facing glue stays in
# shim.c.

def make_hook():
    """Return the callable the runtime invokes for every kernel class load."""
    def on_class_load(name, **_):
        import sys
        sys.stderr.write("[hello-py] hook fired for %s\n" % name)
        return None  # keep original bytes (a real patch would return bytes)

    on_class_load.__name__ = "on_class_load"
    return on_class_load