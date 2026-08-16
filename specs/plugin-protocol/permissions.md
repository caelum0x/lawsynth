# Permissions

Hosts MUST apply least privilege: deny every capability absent from either the
manifest or host policy. filesystem.*, network, and process.execute are
security-sensitive and require host-side path, network, and process isolation;
the protocol cannot make those operations safe by declaration alone.

trusted-native is an explicit trust boundary, not a sandbox. A conforming host
SHOULD disable it by default and MUST require its mandatory process.execute
declaration. No permission prompts, identities, or persistent grant format are
implemented here.
