# Anode KV

This is a prototype of a basic key-value store. This is being developed for educational purposes only!


# Running

TODO: instructions on launching.


# Architecture

The initial architecture has these main components:

- **connection manager**: responsible for accepting connections and managing the metadata associated, and encoding/decoding protocols between the client protocols and internal representations
- **storage manager**: responsible for keeping track of data in memory or on the disk
- **command processor**: responsible for taking commands from the *process manager* and executing them (verify validity, plan how to do it, and orchestrate the execution of the command)
- **process manager**: responsible for taking raw input from the *connection manager* and batching it up into a full command which is ready to be processed.
	- in the future may also be responsible for admission control (throttle things when sever is saturated) and more scheduling; simple for now

