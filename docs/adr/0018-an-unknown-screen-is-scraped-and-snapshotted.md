# 18. An unknown screen is scraped, marked as such, and snapshotted

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion
- **Not built.** An unclaimed screen is silent today, and auto-capture does not exist. Screen text
  covers more than expected, which is why this has not bitten yet.

## Decision

On a screen no reader claims: pull whatever readable text is there, speak it **with a marker
saying it is a fallback**, and auto-capture a deduplicated snapshot.

## Consequences

Normal play becomes the bug report. The player is never left in silence wondering whether pa3ds
has crashed, and the evidence needed to write the missing reader arrives without anyone having to
reproduce anything.
