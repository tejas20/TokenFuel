# Provider connection fix validation

Validated on Windows x64, 6 October 2026.

The previous reqwest client used bundled rustls/webpki roots. Read-only,
unauthenticated probes to both GitHub and Google reproduced
`InvalidCertificate(UnknownIssuer)`. Windows curl could reach the same hosts.
Switching reqwest to native TLS uses Windows Schannel certificate validation,
including the Windows trust store, without disabling certificate checks.

Live quota reads using the configured local sign-ins then succeeded:

- Antigravity: available, four quota pools.
- GitHub Copilot: available, three quota pools.

The temporary diagnostic printed only provider status and pool counts and was
removed after validation. No model requests were sent.

The details panel previously rendered an error paragraph and then rendered the
same message again when there were no limits. It now shows that failure once.
Both provider cases have UI regression tests. A synthetic Copilot Free fixture
checks separate chat/completion quotas, exhausted chat, and monthly reset data;
allowances are read from the response rather than hardcoded by plan.

Antigravity uses Google's press icon and Copilot uses GitHub's Primer Octicon.
Sources and license notices are in `public/providers/README.md`.

Initial connection-fix validation: 35 frontend tests and 41 Rust tests passed; the frontend production
build and optimized Windows executable build passed. The executable is
`target/release/tokenfuel.exe`. Live provider reads verify these local sign-ins,
not every plan or network environment.

The subsequent 0.2.0 release rebuild passed 45 frontend tests and 41 Rust tests,
full Tauri packaging, both installer upgrades and a portable launch. See
[release validation](release-validation-v0.2.0.md) for the current release checks.
