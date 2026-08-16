# Time

Continuous simulation accepts finite `start`, `end`, and positive `step`, then integrates until the requested end time. Discrete simulation uses a positive integer number of updates. The recorded trajectory includes the initial sample.

Scheduled parameter and input changes split a continuous integration interval at their declared time. A change must use a finite time and a known target. This gives a defined piecewise-constant input convention for ordinary simulation requests.

The core World IR does not attach a calendar, timezone, sample-rate unit, or uncertainty distribution to time.
