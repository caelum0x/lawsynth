# Discovery canvas boundary

Discovery is performed by the native CLI or Python binding, not by a TypeScript canvas. A client can collect validated configuration values, launch an application-owned job, and display returned bundle metadata, but it must not fabricate progress, candidates, or equation results.

Expose only implemented controls: state selection, polynomial degree, threshold, `stlsq`/`sr3`, trigonometric/rational features, one derivative method, optional bootstrap, and bounded symbolic depth. Validate these values on both client and execution boundary and record the exact request.

The repository contains no Studio job API, web worker bridge, multi-user canvas, or server deployment. Build those services separately with authentication, authorization, cancellation, and audit logging before connecting the UI to production data.
