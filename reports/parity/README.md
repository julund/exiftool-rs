# Parity report

Run `bash scripts/generate_parity_report.sh` from anywhere in the repository.
It writes the self-contained report to `reports/parity/index.html`.

Generated HTML is intentionally ignored by Git. The `Parity report` GitHub
Actions workflow runs the same script, retains `index.html` as a downloadable
artifact, and deploys it to <https://julund.github.io/exiftool-rs/>.
