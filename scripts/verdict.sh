# **One beat's verdict**, shared by every loop in this harness (bl-05b6).
#
# It was `screens-capture.sh`'s alone until the invocation beat needed the same
# two lines. A second copy of a verdict is how two loops end up disagreeing
# about what a failure is — the whole reason this file exists is that `FAILED`
# has to mean one thing.
FAILED=0
verdict() {            # verdict <pass|fail> <label>
  echo "  $1  $2" | tee -a "$OUT/verdict.txt"
  [ "$1" = fail ] && FAILED=1
  return 0
}
