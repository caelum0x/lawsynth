# Local communication

`LocalComm` is a synchronous in-process publish/subscribe transport. It copies
message objects for storage and each callback. It is a useful adapter point for
a notebook frontend but is not an implementation of Jupyter comms.
