+++
title = "a destroy that catches the app mid-dial hangs GameActivity's native teardown, and the platform kills the process"
created = 1788584440
updated = 1788584440
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Found by the bl-05b6 invocation loop on its first run, and it is the app's rather than the harness's.

Installing the package makes the platform re-apply its overlays. That lands as a configuration change and DESTROYS the activity that is already up — ordinary Android, and every rotation and every locale change does the same. When the destroy catches this app while its wire host is dialling, the main thread hangs in GameActivity's native teardown: NativeCode::~NativeCode waits on a pthread condvar for the app thread to finish, and the app thread is inside a wire read that will not return for the length of a hold. The platform then ANRs the process (the trace's subject was another component of this app waiting 74s on the wedged main thread) and kills it with SIGKILL.

Evidence, from the run that found it: the activity was created, resumed, paused and stopped within two seconds of launch as the overlays landed; the ANR trace's main thread reads

  __futex_wait_ex -> pthread_cond_wait -> onDestroy -> NativeCode::~NativeCode
  -> terminateNativeCode_native -> GameActivity.onDestroy -> Activity.performDestroy

and the process was killed 23 seconds later. The screens walk never meets it because minutes of seeds and taps sit between its install and its first relaunch; the invocation loop met it because its first launch is seconds after the install and the engine it dials ANSWERS, so the read blocks where screens' closed port refuses at once.

What the harness does about it: scripts/invoke.sh launches once, waits for the app to say what it painted, and relaunches onto the settled package — so the loop is not what rediscovers this every run. That is a workaround in the harness and this ball is the defect.

What to decide here: whether the app thread must become interruptible at teardown (the destroy command is queued while a blocking read holds the thread), or whether the wire reads a foot holds need a bound short enough that a destroy is never waiting on one, or whether this is upstream android-activity's (bl-2958 already carries three upstream shims and may be where this one goes). The reads in question are the host's parked invocations read — a 30s hold by the engine's own contract — and the seat lanes beside it (DESIGN §14.1).

A phone that hangs on rotation is not a shippable seat, so this outranks a harness convenience.