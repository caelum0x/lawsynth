# Repositories

Every repository method takes an organization ID. Lookup, list, update, and
soft delete enforce that boundary in the repository rather than trusting a
route handler. Human names are unique only inside an organization; deleted
records are excluded from normal queries.
