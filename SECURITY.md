# Security policy

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private
security advisory flow for `caelum0x/lawsynth`, including a minimal reproduction,
affected revision, impact, and any proposed mitigation. If that flow is not
available, contact the repository owner privately through GitHub.

## Scope and response

The supported security surface is the checked-in source and its documented
local CLI/Python behavior. Hosted APIs, plugins, tenant isolation, and
deployment artifacts are not implemented products and therefore do not have an
operational security SLA. Maintainers will acknowledge and assess reports when
capacity permits; no fixed response or release timeline is promised.

Avoid publishing exploit details until maintainers have had a reasonable chance
to reproduce and coordinate a fix.
