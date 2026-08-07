@echo off
rem v2 on Windows: boot the kernel from versions\ with the native runtime + modules.
rem Usage: run.bat [extra kernel args]. DIST_JAVA_OPTS overrides JVM flags.
setlocal
cd /d "%~dp0"
if not exist launcher\launcher.jar (
    echo launcher.jar missing - build it: javac -d launcher\out src... 1>&2
    exit /b 2
)
java -jar launcher\launcher.jar %*