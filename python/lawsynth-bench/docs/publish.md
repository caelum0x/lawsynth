# Publishing reports

`write_report` writes JSON to a temporary file in the destination directory and atomically replaces the destination. It publishes locally only; remote publishing and credentials are deliberately outside this package.
