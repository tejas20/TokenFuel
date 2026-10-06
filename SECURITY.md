# Security policy

## Supported version

Security fixes target the latest release and `main`. Version 0.2.0 is a prerelease; provider compatibility and native validation limits are documented in its release notes. Older releases may require upgrading.

## Report a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/tejas20/TokenFuel/security/advisories/new) to contact the maintainer, @tejas20. Do not post credentials, account data, exploitation details or sensitive logs in public issues or pull requests.

Describe the affected version/source, impact and reproduction steps using synthetic data where possible. Never include live tokens, cookies or passwords. The maintainer will assess the report and coordinate a fix and disclosure; no response-time guarantee or paid bounty is offered.

## Boundaries

TokenFuel reads supported local sign-ins and contacts provider quota services. Optional app-specific secrets use Windows Credential Manager; browser sign-ins are isolated and temporary. Disconnecting or removing a connection deletes TokenFuel's saved secret, without rewriting the original provider credentials. See [discovery](docs/discovery.md) and the README for data handling and experimental integration limits.

Release binaries are currently unsigned. Published SHA-256 checksums verify download integrity against the release, not publisher identity. There is no automatic updater; install a newer release manually.
