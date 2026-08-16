# Lifecycle

The allowed state machine is Discovered --Validate--> Validated --Start-->
Starting --Ready--> Running --Drain--> Draining --Stop--> Stopped. Stop is also
allowed from Starting and Running; Fail is allowed from every nonterminal state
and yields Failed. All other transitions reject.

Only Running accepts requests. The state machine has no restart, upgrade,
health-check, persistence, or process-control operation. A host owns the actual
startup and shutdown mechanics.
