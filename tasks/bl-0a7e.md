+++
title = "Push tests race the job threads; the 4-core runner fails a_pushed_id_already_held_is_not_started_twice"
created = 1795582733
updated = 1795582734
claimant = "Rabid"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["bug"]
+++
CI / linux on main (bl-1782) fails host::tests::push::a_pushed_id_already_held_is_not_started_twice: left Started(b), right Started(a). Jobs a and b each run on their own thread, so which reports its start first is the scheduler's choice; the test assumed spawn order. The host is sound: an id enters Held in the worker, synchronously, before any thread is spawned, and leaves only on an answered post. Sibling push tests make the same kind of assumption: a 300ms hold standing in for concurrency, 300ms sleeps standing in for a post's answer reaching the worker, and a silence bound racing a post. Fix: wait on events. The pushing host counts a job (Note::Ran) once its post is answered, after the worker is told, which is what Standing::served already documents and gives the tests an event to wait on.