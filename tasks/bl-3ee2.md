+++
title = "a boot-started app process answers 'this app has not finished starting' for device/notify: App.context() is null when BOOT_COMPLETED starts the process, though DESIGN §16.1 promises those tools work boot-started"
created = 1789003454
updated = 1789003454
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
Seen once by lane A11 during make invoke: the process was started by the BOOT_COMPLETED receiver rather than the activity, App.context() was null, so device and notify answered the not-started sentence and no foreground service existed. A re-run cleared it — a race between the receiver and the App's context publication. DESIGN §16.1's table claims device/clipboard_set/notifications work boot-started; make the code keep that promise (the Application subclass publishes its context before any receiver's work runs, or the receiver waits on it) and add the boot-started arm to the invoke harness so it is not a one-run sighting.