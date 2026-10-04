#!bin/bash
#install scrcpy in fedora: sudo dnf copr enable zeno/scrcpy && sudo dnf install scrcpy


scrcpy \
  --max-size 1440 \
  --video-bit-rate 50M \
  --video-codec h265 \
  --video-codec-options profile=1 \
  --max-fps 60