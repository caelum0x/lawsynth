# Capabilities

Declared capabilities are dataset.read, artifact.write, filesystem.read,
filesystem.write, network, process.execute, world.validate, algorithm,
data.adapter, and simulator. They use a deterministically ordered set and
unknown names reject at manifest parse time.

Declaration does not grant access. A host MUST compute an effective set by
requiring the plugin declaration to be a subset of its granted policy before
executing privileged work. The API supplies the subset primitive but does not
perform OS sandboxing or authorization itself.
