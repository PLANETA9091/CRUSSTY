#!/bin/bash
# Crussty demo recording script — runs inside an asciinema pty.
#
# Usage:
#   cd <workdir>                       # fresh empty directory
#   asciinema rec --cols 90 --rows 28 -t "crussty demo" demo.cast \
#     --command "bash <repo>/assets/demo/demo.sh"
#
# Then render the terminal gif (agg, custom theme matching TERM_BG in
# render.py):
#   agg --theme 0c0c0e,d5d8e0,0c0c0e,777777,ff5555,50fa7b,f1fa8c,bd93f9,ff79c6,8be9fd,f8f8f2,343746,ff6e67,5af78e,f4f99d,caa9fa,ff92d0,9aedfe \
#       --speed 2.0 --idle-time-limit 1.2 --font-size 13 \
#       --cols 90 --rows 28 demo.cast demo_terminal.gif
#
# Finally build the desktop-scene gif:
#   python3 <repo>/assets/demo/render.py demo_terminal.gif demo_final.gif
set -u

say() {
  printf '$ '
  for ((i = 0; i < ${#1}; i++)); do
    printf '%s' "${1:$i:1}"
    sleep 0.05
  done
  printf '\n'
  eval "$1"
  local rc=$?
  sleep 0.6
  return $rc
}

export PS1=''

say "crussty --version"
sleep 1.2
say "crussty init --dir my-server"
sleep 1.2
say "ls -F my-server"
printf '\n# cd in, grab the hello module, sign the EULA, boot the server\n'
sleep 1.0
say "cd my-server"
say "crussty install hello"
say "printf 'eula=true\\n' > eula.txt"
printf '\n'
printf '$ crussty run\n'
crussty run &
RUN_PID=$!
# wait for the server to finish booting (first run generates the world)
for _ in $(seq 1 90); do
  grep -q "Done (" my-server/logs/latest.log 2>/dev/null && break
  sleep 2
done
sleep 3
say "crussty stop"
wait $RUN_PID 2>/dev/null
sleep 1.0
# user interrupts back at the prompt, clears the screen, walks away
printf '^C'
sleep 0.8
printf '\n'
sleep 0.6
say "clear"
sleep 1.5
